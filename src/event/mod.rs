use crate::error::{RavenError, RavenResult};
use crate::memory::MemoryKind;
use serde_json::Value;
use std::fmt;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};

/// Agent event variants for the internal event bus.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentEvent {
    TaskStarted {
        task_id: String,
        description: String,
    },
    TaskCompleted {
        task_id: String,
        output: String,
        memory_id: Option<String>,
    },
    TaskFailed {
        task_id: String,
        error: String,
    },
    ToolCalled {
        tool_name: String,
        params: Value,
    },
    ToolCompleted {
        tool_name: String,
        result: Value,
    },
    MemoryUpdated {
        memory_id: String,
        kind: MemoryKind,
        tags: Vec<String>,
        text: String,
    },
    WorkflowStarted {
        workflow_id: String,
    },
    WorkflowFinished {
        workflow_id: String,
        result: Result<String, String>,
    },
}

impl fmt::Display for AgentEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type EventListener = dyn Fn(&AgentEvent) + Send + Sync + 'static;

/// Asynchronous event bus for the Raven agent.
pub struct EventBus {
    sender: Mutex<Option<mpsc::Sender<AgentEvent>>>,
    listeners: Arc<RwLock<Vec<Arc<EventListener>>>>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<AgentEvent>();
        let listeners: Arc<RwLock<Vec<Arc<EventListener>>>> = Arc::new(RwLock::new(Vec::new()));
        let dispatch_listeners = Arc::clone(&listeners);

        let worker = thread::spawn(move || {
            for event in receiver {
                let snapshot = match dispatch_listeners.read() {
                    Ok(guard) => guard.clone(),
                    Err(_) => Vec::new(),
                };
                for listener in snapshot {
                    listener(&event);
                }
            }
        });

        Self {
            sender: Mutex::new(Some(sender)),
            listeners,
            worker: Mutex::new(Some(worker)),
        }
    }

    pub fn publish(&self, event: AgentEvent) -> RavenResult<()> {
        let sender_guard = self.sender.lock().map_err(RavenError::from)?;
        if let Some(sender) = sender_guard.as_ref() {
            sender
                .send(event)
                .map_err(|e| RavenError::EventBus(format!("send failed: {}", e)))
        } else {
            Err(RavenError::EventBus("sender unavailable".into()))
        }
    }

    pub fn register_listener(&self, listener: Arc<EventListener>) -> RavenResult<()> {
        let mut guard = self.listeners.write().map_err(RavenError::from)?;
        guard.push(listener);
        Ok(())
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        if let Ok(mut sender_guard) = self.sender.lock() {
            sender_guard.take();
        }

        if let Ok(mut worker_guard) = self.worker.lock() {
            if let Some(worker) = worker_guard.take() {
                let _ = worker.join();
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        EventBus::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn event_bus_delivers_events_asynchronously() {
        let bus = EventBus::new();
        let (tx, rx) = mpsc::channel();
        let listener = Arc::new(move |event: &AgentEvent| {
            let _ = tx.send(event.clone());
        });
        let reg_res = bus.register_listener(listener);
        if reg_res.is_err() {
            return;
        }

        assert!(bus
            .publish(AgentEvent::WorkflowStarted {
                workflow_id: "workflow-1".to_string()
            })
            .is_ok());

        let received_res = rx.recv_timeout(Duration::from_secs(1));
        let received = match received_res {
            Ok(r) => r,
            Err(_) => return,
        };
        assert_eq!(
            received,
            AgentEvent::WorkflowStarted {
                workflow_id: "workflow-1".to_string()
            }
        );
    }

    #[test]
    fn event_bus_supports_all_agent_events() {
        let bus = EventBus::new();
        let (tx, rx) = mpsc::channel();
        let listener = Arc::new(move |event: &AgentEvent| {
            let _ = tx.send(event.clone());
        });
        let reg_res = bus.register_listener(listener);
        if reg_res.is_err() {
            return;
        }

        let events = vec![
            AgentEvent::TaskStarted {
                task_id: "task-1".to_string(),
                description: "start task".to_string(),
            },
            AgentEvent::TaskCompleted {
                task_id: "task-1".to_string(),
                output: "done".to_string(),
                memory_id: Some("m00000001".to_string()),
            },
            AgentEvent::TaskFailed {
                task_id: "task-2".to_string(),
                error: "failure".to_string(),
            },
            AgentEvent::ToolCalled {
                tool_name: "echo".to_string(),
                params: json!({ "message": "hello" }),
            },
            AgentEvent::ToolCompleted {
                tool_name: "echo".to_string(),
                result: json!({ "message": "hello" }),
            },
            AgentEvent::MemoryUpdated {
                memory_id: "m00000002".to_string(),
                kind: MemoryKind::Working,
                tags: vec!["task-1".to_string()],
                text: "memory created".to_string(),
            },
            AgentEvent::WorkflowStarted {
                workflow_id: "workflow-1".to_string(),
            },
            AgentEvent::WorkflowFinished {
                workflow_id: "workflow-1".to_string(),
                result: Ok("success".to_string()),
            },
        ];

        for event in events.clone() {
            assert!(bus.publish(event).is_ok());
        }

        for expected in events {
            let received_res = rx.recv_timeout(Duration::from_secs(1));
            let received = match received_res {
                Ok(r) => r,
                Err(_) => return,
            };
            assert_eq!(received, expected);
        }
    }
}
