use async_trait::async_trait;
use beater_core::{IdempotencyKey, ProjectId, TenantId, Timestamp};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BusError {
    #[error("bus is at capacity {capacity}")]
    Backpressure { capacity: usize },
    #[error("bus message not found: {0}")]
    NotFound(String),
    #[error("bus mutex poisoned: {0}")]
    Poisoned(String),
    #[error("bus storage error: {0}")]
    Storage(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BusMessage {
    pub message_id: String,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub idempotency_key: IdempotencyKey,
    pub kind: String,
    pub payload: Vec<u8>,
    pub attempts: u32,
    pub max_attempts: u32,
    #[schema(value_type = String, format = DateTime)]
    pub enqueued_at: Timestamp,
}

impl BusMessage {
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        idempotency_key: IdempotencyKey,
        kind: impl Into<String>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            tenant_id,
            project_id,
            idempotency_key,
            kind: kind.into(),
            payload,
            attempts: 0,
            max_attempts: 3,
            enqueued_at: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeadLetter {
    pub message: BusMessage,
    pub reason: String,
    #[schema(value_type = String, format = DateTime)]
    pub failed_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PublishAck {
    pub accepted: bool,
    pub duplicate: bool,
}

impl PublishAck {
    pub fn accepted() -> Self {
        Self {
            accepted: true,
            duplicate: false,
        }
    }

    pub fn duplicate() -> Self {
        Self {
            accepted: false,
            duplicate: true,
        }
    }
}

#[async_trait]
pub trait DurableBus: Send + Sync {
    async fn publish(&self, message: BusMessage) -> Result<PublishAck, BusError>;
    async fn consume_batch(&self, limit: usize) -> Result<Vec<BusMessage>, BusError>;
    async fn consume_kind_batch(
        &self,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<BusMessage>, BusError>;
    async fn consume_scoped_kind_batch(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<BusMessage>, BusError>;
    async fn ack(&self, message: BusMessage) -> Result<(), BusError>;
    async fn retry_or_dlq(&self, message: BusMessage, reason: String) -> Result<(), BusError>;
    async fn replay_dead_letter(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        message_id: &str,
        reset_attempts: bool,
    ) -> Result<PublishAck, BusError>;
    async fn dlq(&self) -> Result<Vec<DeadLetter>, BusError>;
    async fn depth(&self) -> Result<usize, BusError>;
    async fn depth_for_kind(&self, kind: &str) -> Result<usize, BusError>;
}

#[derive(Clone, Debug)]
pub struct InMemoryBus {
    state: Arc<Mutex<BusState>>,
    capacity: usize,
}

#[derive(Debug, Default)]
struct BusState {
    queue: VecDeque<BusMessage>,
    inflight: Vec<BusMessage>,
    dlq: Vec<DeadLetter>,
}

impl InMemoryBus {
    pub fn new(capacity: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(BusState::default())),
            capacity,
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, BusState>, BusError> {
        self.state
            .lock()
            .map_err(|err| BusError::Poisoned(err.to_string()))
    }

    fn active_depth(state: &BusState) -> usize {
        state.queue.len().saturating_add(state.inflight.len())
    }

    fn has_active_duplicate(state: &BusState, message: &BusMessage) -> bool {
        state
            .queue
            .iter()
            .chain(state.inflight.iter())
            .any(|queued| {
                queued.tenant_id == message.tenant_id
                    && queued.project_id == message.project_id
                    && queued.kind == message.kind
                    && queued.idempotency_key == message.idempotency_key
            })
    }
}

#[async_trait]
impl DurableBus for InMemoryBus {
    async fn publish(&self, message: BusMessage) -> Result<PublishAck, BusError> {
        let mut state = self.lock()?;
        if Self::has_active_duplicate(&state, &message) {
            return Ok(PublishAck::duplicate());
        }
        if Self::active_depth(&state) >= self.capacity {
            return Err(BusError::Backpressure {
                capacity: self.capacity,
            });
        }
        state.queue.push_back(message);
        Ok(PublishAck::accepted())
    }

    async fn consume_batch(&self, limit: usize) -> Result<Vec<BusMessage>, BusError> {
        let mut state = self.lock()?;
        let mut messages = Vec::new();
        for _ in 0..limit {
            if let Some(message) = state.queue.pop_front() {
                state.inflight.push(message.clone());
                messages.push(message);
            } else {
                break;
            }
        }
        Ok(messages)
    }

    async fn consume_kind_batch(
        &self,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<BusMessage>, BusError> {
        let mut state = self.lock()?;
        let mut messages = Vec::new();
        let mut index = 0;
        while messages.len() < limit && index < state.queue.len() {
            if state.queue[index].kind == kind {
                if let Some(message) = state.queue.remove(index) {
                    state.inflight.push(message.clone());
                    messages.push(message);
                }
            } else {
                index += 1;
            }
        }
        Ok(messages)
    }

    async fn consume_scoped_kind_batch(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<BusMessage>, BusError> {
        let mut state = self.lock()?;
        let mut messages = Vec::new();
        let mut index = 0;
        while messages.len() < limit && index < state.queue.len() {
            let queued = &state.queue[index];
            if queued.tenant_id == *tenant_id
                && queued.project_id == *project_id
                && queued.kind == kind
            {
                if let Some(message) = state.queue.remove(index) {
                    state.inflight.push(message.clone());
                    messages.push(message);
                }
            } else {
                index += 1;
            }
        }
        Ok(messages)
    }

    async fn ack(&self, message: BusMessage) -> Result<(), BusError> {
        let mut state = self.lock()?;
        state
            .inflight
            .retain(|inflight| inflight.message_id != message.message_id);
        Ok(())
    }

    async fn retry_or_dlq(&self, mut message: BusMessage, reason: String) -> Result<(), BusError> {
        let mut state = self.lock()?;
        state
            .inflight
            .retain(|inflight| inflight.message_id != message.message_id);
        message.attempts = message.attempts.saturating_add(1);
        if message.attempts >= message.max_attempts {
            state.dlq.push(DeadLetter {
                message,
                reason,
                failed_at: Utc::now(),
            });
            return Ok(());
        }
        if Self::active_depth(&state) >= self.capacity {
            state.dlq.push(DeadLetter {
                message,
                reason: format!("retry queue full after failure: {reason}"),
                failed_at: Utc::now(),
            });
            return Ok(());
        }
        state.queue.push_back(message);
        Ok(())
    }

    async fn replay_dead_letter(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        message_id: &str,
        reset_attempts: bool,
    ) -> Result<PublishAck, BusError> {
        let mut state = self.lock()?;
        let index = state
            .dlq
            .iter()
            .position(|dead_letter| {
                dead_letter.message.tenant_id == *tenant_id
                    && dead_letter.message.project_id == *project_id
                    && dead_letter.message.message_id == message_id
            })
            .ok_or_else(|| BusError::NotFound(message_id.to_string()))?;
        let mut message = state.dlq[index].message.clone();
        if reset_attempts {
            message.attempts = 0;
        }
        message.enqueued_at = Utc::now();
        if Self::has_active_duplicate(&state, &message) {
            return Ok(PublishAck::duplicate());
        }
        if Self::active_depth(&state) >= self.capacity {
            return Err(BusError::Backpressure {
                capacity: self.capacity,
            });
        }
        state.dlq.remove(index);
        state.queue.push_back(message);
        Ok(PublishAck::accepted())
    }

    async fn dlq(&self) -> Result<Vec<DeadLetter>, BusError> {
        let state = self.lock()?;
        Ok(state.dlq.clone())
    }

    async fn depth(&self) -> Result<usize, BusError> {
        let state = self.lock()?;
        Ok(Self::active_depth(&state))
    }

    async fn depth_for_kind(&self, kind: &str) -> Result<usize, BusError> {
        let state = self.lock()?;
        Ok(state
            .queue
            .iter()
            .chain(state.inflight.iter())
            .filter(|message| message.kind == kind)
            .count())
    }
}

#[derive(Clone)]
pub struct SqliteDurableBus {
    connection: Arc<Mutex<Connection>>,
    capacity: usize,
}

impl SqliteDurableBus {
    pub fn in_memory(capacity: usize) -> Result<Self, BusError> {
        let connection =
            Connection::open_in_memory().map_err(|err| BusError::Storage(err.to_string()))?;
        let bus = Self {
            connection: Arc::new(Mutex::new(connection)),
            capacity,
        };
        bus.init()?;
        Ok(bus)
    }

    pub fn open(path: impl AsRef<Path>, capacity: usize) -> Result<Self, BusError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| BusError::Storage(err.to_string()))?;
        }
        let connection =
            Connection::open(path).map_err(|err| BusError::Storage(err.to_string()))?;
        let bus = Self {
            connection: Arc::new(Mutex::new(connection)),
            capacity,
        };
        bus.init()?;
        Ok(bus)
    }

    fn init(&self) -> Result<(), BusError> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS queue_messages (
                    message_id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    enqueued_at TEXT NOT NULL,
                    message_json TEXT NOT NULL,
                    UNIQUE (tenant_id, project_id, kind, idempotency_key)
                );

                CREATE INDEX IF NOT EXISTS idx_queue_order
                ON queue_messages (enqueued_at, message_id);

                CREATE TABLE IF NOT EXISTS inflight_messages (
                    message_id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    leased_at TEXT NOT NULL,
                    message_json TEXT NOT NULL,
                    UNIQUE (tenant_id, project_id, kind, idempotency_key)
                );

                CREATE INDEX IF NOT EXISTS idx_inflight_kind
                ON inflight_messages (kind, leased_at, message_id);

                CREATE TABLE IF NOT EXISTS dead_letters (
                    message_id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    failed_at TEXT NOT NULL,
                    dead_letter_json TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_dead_letters_order
                ON dead_letters (failed_at, message_id);
                "#,
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Self::recover_inflight(&connection)?;
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, BusError> {
        self.connection
            .lock()
            .map_err(|err| BusError::Poisoned(err.to_string()))
    }

    fn queue_depth(connection: &Connection) -> Result<usize, BusError> {
        connection
            .query_row("SELECT COUNT(*) FROM queue_messages", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|count| count as usize)
            .map_err(|err| BusError::Storage(err.to_string()))
    }

    fn queue_depth_for_kind(connection: &Connection, kind: &str) -> Result<usize, BusError> {
        connection
            .query_row(
                "SELECT COUNT(*) FROM queue_messages WHERE kind = ?1",
                params![kind],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count as usize)
            .map_err(|err| BusError::Storage(err.to_string()))
    }

    fn inflight_depth(connection: &Connection) -> Result<usize, BusError> {
        connection
            .query_row("SELECT COUNT(*) FROM inflight_messages", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|count| count as usize)
            .map_err(|err| BusError::Storage(err.to_string()))
    }

    fn inflight_depth_for_kind(connection: &Connection, kind: &str) -> Result<usize, BusError> {
        connection
            .query_row(
                "SELECT COUNT(*) FROM inflight_messages WHERE kind = ?1",
                params![kind],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count as usize)
            .map_err(|err| BusError::Storage(err.to_string()))
    }

    fn active_depth(connection: &Connection) -> Result<usize, BusError> {
        Ok(Self::queue_depth(connection)?.saturating_add(Self::inflight_depth(connection)?))
    }

    fn active_depth_for_kind(connection: &Connection, kind: &str) -> Result<usize, BusError> {
        Ok(Self::queue_depth_for_kind(connection, kind)?
            .saturating_add(Self::inflight_depth_for_kind(connection, kind)?))
    }

    fn inflight_exists(connection: &Connection, message_id: &str) -> Result<bool, BusError> {
        connection
            .query_row(
                "SELECT COUNT(*) FROM inflight_messages WHERE message_id = ?1",
                params![message_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(|err| BusError::Storage(err.to_string()))
    }

    fn active_duplicate_exists(
        connection: &Connection,
        message: &BusMessage,
    ) -> Result<bool, BusError> {
        connection
            .query_row(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM queue_messages
                    WHERE tenant_id = ?1 AND project_id = ?2 AND kind = ?3 AND idempotency_key = ?4
                    UNION ALL
                    SELECT 1 FROM inflight_messages
                    WHERE tenant_id = ?1 AND project_id = ?2 AND kind = ?3 AND idempotency_key = ?4
                )
                "#,
                params![
                    message.tenant_id.as_str(),
                    message.project_id.as_str(),
                    message.kind.as_str(),
                    message.idempotency_key.as_str()
                ],
                |row| row.get::<_, i64>(0),
            )
            .map(|exists| exists != 0)
            .map_err(|err| BusError::Storage(err.to_string()))
    }

    fn recover_inflight(connection: &Connection) -> Result<(), BusError> {
        let mut messages = Vec::new();
        {
            let mut statement = connection
                .prepare(
                    r#"
                    SELECT message_json
                    FROM inflight_messages
                    ORDER BY leased_at ASC, message_id ASC
                    "#,
                )
                .map_err(|err| BusError::Storage(err.to_string()))?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|err| BusError::Storage(err.to_string()))?;
            for row in rows {
                let json = row.map_err(|err| BusError::Storage(err.to_string()))?;
                messages.push(
                    serde_json::from_str::<BusMessage>(&json)
                        .map_err(|err| BusError::Storage(err.to_string()))?,
                );
            }
        }
        for message in messages {
            Self::insert_message(connection, &message)?;
        }
        connection
            .execute("DELETE FROM inflight_messages", [])
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_message(
        connection: &Connection,
        message: &BusMessage,
    ) -> Result<PublishAck, BusError> {
        let message_json =
            serde_json::to_string(message).map_err(|err| BusError::Storage(err.to_string()))?;
        let changed = connection
            .execute(
                r#"
                INSERT OR IGNORE INTO queue_messages
                  (message_id, tenant_id, project_id, idempotency_key, kind, enqueued_at, message_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    message.message_id.as_str(),
                    message.tenant_id.as_str(),
                    message.project_id.as_str(),
                    message.idempotency_key.as_str(),
                    message.kind.as_str(),
                    message.enqueued_at.to_rfc3339(),
                    message_json,
                ],
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
        if changed == 0 {
            Ok(PublishAck::duplicate())
        } else {
            Ok(PublishAck::accepted())
        }
    }

    fn insert_dead_letter(
        connection: &Connection,
        dead_letter: &DeadLetter,
    ) -> Result<(), BusError> {
        let json =
            serde_json::to_string(dead_letter).map_err(|err| BusError::Storage(err.to_string()))?;
        connection
            .execute(
                r#"
                INSERT OR REPLACE INTO dead_letters
                  (message_id, tenant_id, project_id, idempotency_key, kind, failed_at, dead_letter_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    dead_letter.message.message_id.as_str(),
                    dead_letter.message.tenant_id.as_str(),
                    dead_letter.message.project_id.as_str(),
                    dead_letter.message.idempotency_key.as_str(),
                    dead_letter.message.kind.as_str(),
                    dead_letter.failed_at.to_rfc3339(),
                    json,
                ],
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(())
    }

    fn delete_dead_letter(connection: &Connection, message_id: &str) -> Result<(), BusError> {
        connection
            .execute(
                "DELETE FROM dead_letters WHERE message_id = ?1",
                params![message_id],
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_inflight(connection: &Connection, message: &BusMessage) -> Result<(), BusError> {
        let message_json =
            serde_json::to_string(message).map_err(|err| BusError::Storage(err.to_string()))?;
        connection
            .execute(
                r#"
                INSERT INTO inflight_messages
                  (message_id, tenant_id, project_id, idempotency_key, kind, leased_at, message_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    message.message_id.as_str(),
                    message.tenant_id.as_str(),
                    message.project_id.as_str(),
                    message.idempotency_key.as_str(),
                    message.kind.as_str(),
                    Utc::now().to_rfc3339(),
                    message_json,
                ],
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(())
    }

    fn delete_inflight(connection: &Connection, message_id: &str) -> Result<(), BusError> {
        connection
            .execute(
                "DELETE FROM inflight_messages WHERE message_id = ?1",
                params![message_id],
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl DurableBus for SqliteDurableBus {
    async fn publish(&self, message: BusMessage) -> Result<PublishAck, BusError> {
        let connection = self.lock()?;
        if Self::active_duplicate_exists(&connection, &message)? {
            return Ok(PublishAck::duplicate());
        }
        if Self::active_depth(&connection)? >= self.capacity {
            return Err(BusError::Backpressure {
                capacity: self.capacity,
            });
        }
        Self::insert_message(&connection, &message)
    }

    async fn consume_batch(&self, limit: usize) -> Result<Vec<BusMessage>, BusError> {
        self.consume_batch_inner(ConsumeFilter::All, limit).await
    }

    async fn consume_kind_batch(
        &self,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<BusMessage>, BusError> {
        self.consume_batch_inner(ConsumeFilter::Kind { kind }, limit)
            .await
    }

    async fn consume_scoped_kind_batch(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        kind: &str,
        limit: usize,
    ) -> Result<Vec<BusMessage>, BusError> {
        self.consume_batch_inner(
            ConsumeFilter::ScopedKind {
                tenant_id,
                project_id,
                kind,
            },
            limit,
        )
        .await
    }

    async fn ack(&self, message: BusMessage) -> Result<(), BusError> {
        let connection = self.lock()?;
        Self::delete_inflight(&connection, &message.message_id)
    }

    async fn retry_or_dlq(&self, mut message: BusMessage, reason: String) -> Result<(), BusError> {
        let mut connection = self.lock()?;
        let tx = connection
            .transaction()
            .map_err(|err| BusError::Storage(err.to_string()))?;
        let source_is_inflight = Self::inflight_exists(&tx, &message.message_id)?;
        message.attempts = message.attempts.saturating_add(1);
        if message.attempts >= message.max_attempts {
            let dead_letter = DeadLetter {
                message,
                reason,
                failed_at: Utc::now(),
            };
            Self::insert_dead_letter(&tx, &dead_letter)?;
            Self::delete_inflight(&tx, &dead_letter.message.message_id)?;
            tx.commit()
                .map_err(|err| BusError::Storage(err.to_string()))?;
            return Ok(());
        }
        let active_after_source_removal =
            Self::active_depth(&tx)?.saturating_sub(usize::from(source_is_inflight));
        if active_after_source_removal >= self.capacity {
            let dead_letter = DeadLetter {
                message,
                reason: format!("retry queue full after failure: {reason}"),
                failed_at: Utc::now(),
            };
            Self::insert_dead_letter(&tx, &dead_letter)?;
            Self::delete_inflight(&tx, &dead_letter.message.message_id)?;
            tx.commit()
                .map_err(|err| BusError::Storage(err.to_string()))?;
            return Ok(());
        }
        Self::insert_message(&tx, &message)?;
        Self::delete_inflight(&tx, &message.message_id)?;
        tx.commit()
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(())
    }

    async fn replay_dead_letter(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        message_id: &str,
        reset_attempts: bool,
    ) -> Result<PublishAck, BusError> {
        let mut connection = self.lock()?;
        let tx = connection
            .transaction()
            .map_err(|err| BusError::Storage(err.to_string()))?;
        let dead_letter_json = tx
            .query_row(
                r#"
                SELECT dead_letter_json
                FROM dead_letters
                WHERE tenant_id = ?1 AND project_id = ?2 AND message_id = ?3
                "#,
                params![tenant_id.as_str(), project_id.as_str(), message_id],
                |row| row.get::<_, String>(0),
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => BusError::NotFound(message_id.to_string()),
                other => BusError::Storage(other.to_string()),
            })?;
        let dead_letter = serde_json::from_str::<DeadLetter>(&dead_letter_json)
            .map_err(|err| BusError::Storage(err.to_string()))?;
        let mut message = dead_letter.message;
        if reset_attempts {
            message.attempts = 0;
        }
        message.enqueued_at = Utc::now();
        if Self::active_duplicate_exists(&tx, &message)? {
            return Ok(PublishAck::duplicate());
        }
        if Self::active_depth(&tx)? >= self.capacity {
            return Err(BusError::Backpressure {
                capacity: self.capacity,
            });
        }
        let ack = Self::insert_message(&tx, &message)?;
        if ack.accepted {
            Self::delete_dead_letter(&tx, message_id)?;
        }
        tx.commit()
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(ack)
    }

    async fn dlq(&self) -> Result<Vec<DeadLetter>, BusError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT dead_letter_json
                FROM dead_letters
                ORDER BY failed_at ASC, message_id ASC
                "#,
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|err| BusError::Storage(err.to_string()))?;
        let mut dead_letters = Vec::new();
        for row in rows {
            let json = row.map_err(|err| BusError::Storage(err.to_string()))?;
            dead_letters.push(
                serde_json::from_str::<DeadLetter>(&json)
                    .map_err(|err| BusError::Storage(err.to_string()))?,
            );
        }
        Ok(dead_letters)
    }

    async fn depth(&self) -> Result<usize, BusError> {
        let connection = self.lock()?;
        Self::active_depth(&connection)
    }

    async fn depth_for_kind(&self, kind: &str) -> Result<usize, BusError> {
        let connection = self.lock()?;
        Self::active_depth_for_kind(&connection, kind)
    }
}

#[derive(Clone, Copy)]
enum ConsumeFilter<'a> {
    All,
    Kind {
        kind: &'a str,
    },
    ScopedKind {
        tenant_id: &'a TenantId,
        project_id: &'a ProjectId,
        kind: &'a str,
    },
}

impl SqliteDurableBus {
    async fn consume_batch_inner(
        &self,
        filter: ConsumeFilter<'_>,
        limit: usize,
    ) -> Result<Vec<BusMessage>, BusError> {
        let mut connection = self.lock()?;
        let tx = connection
            .transaction()
            .map_err(|err| BusError::Storage(err.to_string()))?;
        let selected = {
            let mut statement = match filter {
                ConsumeFilter::Kind { .. } => tx.prepare(
                    r#"
                    SELECT message_id, message_json
                    FROM queue_messages
                    WHERE kind = ?2
                    ORDER BY enqueued_at ASC, message_id ASC
                    LIMIT ?1
                    "#,
                ),
                ConsumeFilter::ScopedKind { .. } => tx.prepare(
                    r#"
                    SELECT message_id, message_json
                    FROM queue_messages
                    WHERE tenant_id = ?2 AND project_id = ?3 AND kind = ?4
                    ORDER BY enqueued_at ASC, message_id ASC
                    LIMIT ?1
                    "#,
                ),
                ConsumeFilter::All => tx.prepare(
                    r#"
                    SELECT message_id, message_json
                    FROM queue_messages
                    ORDER BY enqueued_at ASC, message_id ASC
                    LIMIT ?1
                    "#,
                ),
            }
            .map_err(|err| BusError::Storage(err.to_string()))?;
            let mut selected = Vec::new();
            match filter {
                ConsumeFilter::Kind { kind } => {
                    let rows = statement
                        .query_map(params![limit as i64, kind], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                        })
                        .map_err(|err| BusError::Storage(err.to_string()))?;
                    for row in rows {
                        selected.push(row.map_err(|err| BusError::Storage(err.to_string()))?);
                    }
                }
                ConsumeFilter::ScopedKind {
                    tenant_id,
                    project_id,
                    kind,
                } => {
                    let rows = statement
                        .query_map(
                            params![limit as i64, tenant_id.as_str(), project_id.as_str(), kind],
                            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                        )
                        .map_err(|err| BusError::Storage(err.to_string()))?;
                    for row in rows {
                        selected.push(row.map_err(|err| BusError::Storage(err.to_string()))?);
                    }
                }
                ConsumeFilter::All => {
                    let rows = statement
                        .query_map(params![limit as i64], |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                        })
                        .map_err(|err| BusError::Storage(err.to_string()))?;
                    for row in rows {
                        selected.push(row.map_err(|err| BusError::Storage(err.to_string()))?);
                    }
                }
            }
            Ok::<_, BusError>(selected)
        }?;

        let mut messages = Vec::with_capacity(selected.len());
        for (message_id, message_json) in selected {
            let message = serde_json::from_str::<BusMessage>(&message_json)
                .map_err(|err| BusError::Storage(err.to_string()))?;
            Self::insert_inflight(&tx, &message)?;
            tx.execute(
                "DELETE FROM queue_messages WHERE message_id = ?1",
                params![message_id],
            )
            .map_err(|err| BusError::Storage(err.to_string()))?;
            messages.push(message);
        }
        tx.commit()
            .map_err(|err| BusError::Storage(err.to_string()))?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bounded_bus_applies_backpressure() {
        let bus = InMemoryBus::new(1);
        let first = fixture_message("one");
        let second = fixture_message("two");

        assert_eq!(bus.publish(first).await, Ok(PublishAck::accepted()));
        assert!(matches!(
            bus.publish(second).await,
            Err(BusError::Backpressure { capacity: 1 })
        ));
    }

    #[tokio::test]
    async fn in_memory_bus_dedupes_publishes_and_consumes_one_kind() {
        let bus = InMemoryBus::new(8);
        let trace_write = fixture_message("trace.write_batch");
        let duplicate = trace_write.clone();
        let downstream = fixture_message("trace.ingested");

        assert_eq!(
            bus.publish(trace_write.clone()).await,
            Ok(PublishAck::accepted())
        );
        assert_eq!(bus.publish(duplicate).await, Ok(PublishAck::duplicate()));
        assert_eq!(
            bus.publish(downstream.clone()).await,
            Ok(PublishAck::accepted())
        );

        let consumed = bus
            .consume_kind_batch("trace.write_batch", 10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(consumed, vec![trace_write]);
        assert_eq!(bus.depth().await, Ok(2));
        assert_eq!(bus.depth_for_kind("trace.write_batch").await, Ok(1));
        bus.ack(consumed[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bus.depth_for_kind("trace.write_batch").await, Ok(0));
        assert_eq!(bus.depth_for_kind("trace.ingested").await, Ok(1));
        let remaining = bus
            .consume_batch(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(remaining, vec![downstream]);
        bus.ack(remaining[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
    }

    #[tokio::test]
    async fn poison_messages_move_to_dlq_without_blocking_queue() {
        let bus = InMemoryBus::new(8);
        let mut poison = fixture_message("poison");
        poison.max_attempts = 2;
        let healthy = fixture_message("healthy");

        bus.publish(poison)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        bus.publish(healthy)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let mut batch = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(batch.len(), 1);
        let poison = batch.remove(0);
        bus.retry_or_dlq(poison, "invalid schema".to_string())
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let mut batch = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let healthy = batch.remove(0);
        assert_eq!(healthy.kind, "healthy");
        bus.ack(healthy).await.unwrap_or_else(|err| panic!("{err}"));

        let mut batch = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let poison = batch.remove(0);
        bus.retry_or_dlq(poison, "invalid schema".to_string())
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let dlq = bus.dlq().await.unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(dlq.len(), 1);
        assert_eq!(dlq[0].reason, "invalid schema");
        assert_eq!(bus.depth().await, Ok(0));
    }

    #[tokio::test]
    async fn in_memory_bus_replays_scoped_dead_letter_with_reset_attempts() {
        let bus = InMemoryBus::new(8);
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let other_tenant = TenantId::new("other-tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut message = scoped_fixture_message(&tenant, &project, "transient", "replay");
        message.max_attempts = 1;

        bus.publish(message)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let mut batch = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let failed = batch.remove(0);
        bus.retry_or_dlq(failed, "transient failure".to_string())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let dlq = bus.dlq().await.unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(dlq.len(), 1);
        let message_id = dlq[0].message.message_id.clone();
        assert_eq!(dlq[0].message.attempts, 1);

        assert!(matches!(
            bus.replay_dead_letter(&other_tenant, &project, &message_id, true)
                .await,
            Err(BusError::NotFound(_))
        ));
        assert_eq!(
            bus.replay_dead_letter(&tenant, &project, &message_id, true)
                .await,
            Ok(PublishAck::accepted())
        );
        assert!(bus
            .dlq()
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .is_empty());
        let replayed = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].message_id, message_id);
        assert_eq!(replayed[0].attempts, 0);
    }

    #[tokio::test]
    async fn sqlite_bus_persists_queue_across_reopen_and_dedupes_publishes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("bus.sqlite");
        let bus = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let first = fixture_message("persisted");
        let duplicate = first.clone();

        bus.publish(first.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let duplicate_ack = bus
            .publish(duplicate)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(duplicate_ack, PublishAck::duplicate());
        assert_eq!(bus.depth().await, Ok(1));
        drop(bus);

        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reopened.depth().await, Ok(1));
        let batch = reopened
            .consume_batch(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(batch, vec![first]);
        reopened
            .ack(batch[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reopened.depth().await, Ok(0));
    }

    #[tokio::test]
    async fn sqlite_bus_recovers_unacked_inflight_messages_on_reopen() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("bus.sqlite");
        let bus = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let message = fixture_message("crash-safe");

        bus.publish(message.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let consumed = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(consumed, vec![message.clone()]);
        assert_eq!(bus.depth().await, Ok(1));
        assert_eq!(bus.depth_for_kind("crash-safe").await, Ok(1));
        drop(bus);

        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reopened.depth().await, Ok(1));
        let recovered = reopened
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(recovered, vec![message]);
        reopened
            .ack(recovered[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        drop(reopened);

        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reopened.depth().await, Ok(0));
    }

    #[tokio::test]
    async fn sqlite_retry_insert_failure_leaves_inflight_recoverable() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("bus.sqlite");
        let bus = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let message = fixture_message("retry-insert-fails");

        bus.publish(message.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let consumed = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(consumed, vec![message.clone()]);
        install_abort_trigger(&bus, "fail_retry_insert", "queue_messages");

        let Err(error) = bus
            .retry_or_dlq(message.clone(), "transient failure".to_string())
            .await
        else {
            panic!("retry_or_dlq should fail when the queue insert is aborted");
        };
        assert!(
            matches!(error, BusError::Storage(message) if message.contains("forced queue_messages insert failure"))
        );
        drop(bus);

        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reopened.depth().await, Ok(1));
        assert!(reopened
            .dlq()
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .is_empty());
        let recovered = reopened
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].message_id, message.message_id);
        assert_eq!(recovered[0].attempts, 0);
    }

    #[tokio::test]
    async fn sqlite_dlq_insert_failure_leaves_inflight_recoverable() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("bus.sqlite");
        let bus = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let mut message = fixture_message("dlq-insert-fails");
        message.max_attempts = 1;

        bus.publish(message.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let consumed = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(consumed, vec![message.clone()]);
        install_abort_trigger(&bus, "fail_dlq_insert", "dead_letters");

        let Err(error) = bus
            .retry_or_dlq(message.clone(), "poison failure".to_string())
            .await
        else {
            panic!("retry_or_dlq should fail when the dead-letter insert is aborted");
        };
        assert!(
            matches!(error, BusError::Storage(message) if message.contains("forced dead_letters insert failure"))
        );
        drop(bus);

        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reopened.depth().await, Ok(1));
        assert!(reopened
            .dlq()
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .is_empty());
        let recovered = reopened
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].message_id, message.message_id);
        assert_eq!(recovered[0].attempts, 0);
    }

    #[tokio::test]
    async fn sqlite_bus_consumes_one_kind_without_stealing_other_lanes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("bus.sqlite");
        let bus = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let trace_write = fixture_message("trace.write_batch");
        let downstream = fixture_message("trace.ingested");

        bus.publish(downstream.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        bus.publish(trace_write.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let consumed = bus
            .consume_kind_batch("trace.write_batch", 10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(consumed, vec![trace_write]);
        bus.ack(consumed[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bus.depth_for_kind("trace.write_batch").await, Ok(0));
        assert_eq!(bus.depth_for_kind("trace.ingested").await, Ok(1));

        drop(bus);
        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let remaining = reopened
            .consume_batch(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(remaining, vec![downstream]);
    }

    #[tokio::test]
    async fn scoped_kind_consumption_preserves_other_tenants() {
        let bus = InMemoryBus::new(8);
        let tenant_a = TenantId::new("tenant-a").unwrap_or_else(|err| panic!("{err}"));
        let tenant_b = TenantId::new("tenant-b").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let a_message = scoped_fixture_message(&tenant_a, &project, "trace.write_batch", "a");
        let b_message = scoped_fixture_message(&tenant_b, &project, "trace.write_batch", "b");

        bus.publish(a_message.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        bus.publish(b_message.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let consumed = bus
            .consume_scoped_kind_batch(&tenant_a, &project, "trace.write_batch", 10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(consumed, vec![a_message]);
        bus.ack(consumed[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bus.depth_for_kind("trace.write_batch").await, Ok(1));
        let remaining = bus
            .consume_batch(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(remaining, vec![b_message]);
        bus.ack(remaining[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
    }

    #[tokio::test]
    async fn sqlite_bus_persists_retry_attempts_and_dlq_across_reopen() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("bus.sqlite");
        let bus = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let mut poison = fixture_message("poison-sqlite");
        poison.max_attempts = 2;

        bus.publish(poison)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let mut batch = bus
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let poison = batch.remove(0);
        bus.retry_or_dlq(poison, "invalid schema".to_string())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        drop(bus);

        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        let mut batch = reopened
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(batch[0].attempts, 1);
        let poison = batch.remove(0);
        reopened
            .retry_or_dlq(poison, "invalid schema".to_string())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        drop(reopened);

        let reopened = SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(reopened.depth().await, Ok(0));
        let dlq = reopened.dlq().await.unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(dlq.len(), 1);
        assert_eq!(dlq[0].reason, "invalid schema");
        assert_eq!(dlq[0].message.attempts, 2);
        assert_eq!(dlq[0].message.kind, "poison-sqlite");
        let message_id = dlq[0].message.message_id.clone();
        assert_eq!(
            reopened
                .replay_dead_letter(
                    &TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                    &ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                    &message_id,
                    true,
                )
                .await,
            Ok(PublishAck::accepted())
        );
        assert_eq!(reopened.depth().await, Ok(1));
        assert!(reopened
            .dlq()
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .is_empty());
        let replayed = reopened
            .consume_batch(1)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].message_id, message_id);
        assert_eq!(replayed[0].attempts, 0);
        reopened
            .ack(replayed[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
    }

    fn fixture_message(kind: &str) -> BusMessage {
        BusMessage::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            IdempotencyKey::new(format!("key-{kind}")).unwrap_or_else(|err| panic!("{err}")),
            kind,
            kind.as_bytes().to_vec(),
        )
    }

    fn scoped_fixture_message(
        tenant_id: &TenantId,
        project_id: &ProjectId,
        kind: &str,
        key: &str,
    ) -> BusMessage {
        BusMessage::new(
            tenant_id.clone(),
            project_id.clone(),
            IdempotencyKey::new(format!("key-{key}")).unwrap_or_else(|err| panic!("{err}")),
            kind,
            key.as_bytes().to_vec(),
        )
    }

    fn install_abort_trigger(bus: &SqliteDurableBus, trigger_name: &str, table_name: &str) {
        let connection = bus.lock().unwrap_or_else(|err| panic!("{err}"));
        connection
            .execute_batch(&format!(
                r#"
                CREATE TEMP TRIGGER {trigger_name}
                BEFORE INSERT ON {table_name}
                BEGIN
                  SELECT RAISE(ABORT, 'forced {table_name} insert failure');
                END;
                "#
            ))
            .unwrap_or_else(|err| panic!("{err}"));
    }

    // ---------------------------------------------------------------------------
    // Pluggability proof: the same round-trip test runs through any DurableBus
    // implementation, proving the trait is the correct seam.
    // ---------------------------------------------------------------------------

    /// Shared publish → consume → ack round-trip exercised via the trait object.
    ///
    /// A second backend that implements `DurableBus` passes with zero changes to
    /// the callers; only this helper needs a new call-site.
    async fn trait_round_trip(bus: std::sync::Arc<dyn DurableBus>) {
        let msg = fixture_message("kind.alpha");

        // publish is accepted and message appears in depth counts
        let ack = bus
            .publish(msg.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(ack, PublishAck::accepted());
        assert_eq!(bus.depth().await.unwrap(), 1);
        assert_eq!(bus.depth_for_kind("kind.alpha").await.unwrap(), 1);

        // kind filter does not bleed into unrelated lanes
        assert_eq!(bus.depth_for_kind("kind.beta").await.unwrap(), 0);

        // consume moves the message to inflight — depth still 1 (queue+inflight)
        let batch = bus
            .consume_batch(10)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].message_id, msg.message_id);
        assert_eq!(bus.depth().await.unwrap(), 1);

        // ack removes from inflight — depth drops to zero, DLQ empty
        bus.ack(batch[0].clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bus.depth().await.unwrap(), 0);
        assert!(bus
            .dlq()
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .is_empty());
    }

    /// The in-memory backend satisfies the `DurableBus` trait object.
    #[tokio::test]
    async fn backend_pluggability_in_memory_bus() {
        let bus: std::sync::Arc<dyn DurableBus> = std::sync::Arc::new(InMemoryBus::new(8));
        trait_round_trip(bus).await;
    }

    /// The SQLite-backed durable bus satisfies the same `DurableBus` trait object,
    /// proving that a second backend (NATS, Kafka, …) can be wired without touching
    /// any caller.
    #[tokio::test]
    async fn backend_pluggability_sqlite_durable_bus() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("bus.sqlite");
        let bus: std::sync::Arc<dyn DurableBus> =
            std::sync::Arc::new(SqliteDurableBus::open(&path, 8).unwrap_or_else(|err| panic!("{err}")));
        trait_round_trip(bus).await;
    }
}
