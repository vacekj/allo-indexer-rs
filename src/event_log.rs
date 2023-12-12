use crate::event_handling::Event;

pub struct InMemoryEventLog {
    pub events: Vec<Event>,
}

pub trait EventLog {
    fn append(&mut self, event: Event);
    fn range(&self, start: usize, end: usize) -> (&[Event], usize);
}

impl EventLog for InMemoryEventLog {
    fn append(&mut self, event: Event) {
        self.events.push(event)
    }

    fn range(&self, chunk_start: usize, chunk_length: usize) -> (&[Event], usize) {
        let desired_end = chunk_start + chunk_length;
        let actual_end = if desired_end > self.events.len() {
            self.events.len()
        } else {
            desired_end
        };
        let range = &self.events[chunk_start..actual_end];
        (range, actual_end)
    }
}

impl InMemoryEventLog {
    pub fn new() -> Self {
        InMemoryEventLog { events: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_handling::{EventPayload, MetaPtr};

    use super::*;

    #[test]
    fn test_requesting_range_from_empty_log_returns_zero_index_and_empty_event_vector() {
        let event_log = InMemoryEventLog { events: Vec::new() };

        let start = 0;
        let (_, next_event_index) = event_log.range(start, 2);

        assert_eq!(next_event_index, 0);
    }

    #[test]
    fn test_requesting_range_past_end_returns_last_index_plus_one_and_empty_event_vector() {
        let mut event_log = InMemoryEventLog { events: Vec::new() };
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        });

        let (_, next_event_index) = event_log.range(0, 999);

        assert_eq!(next_event_index, 1);
    }

    #[test]
    fn test_requesting_range_returns_chunk_of_events_and_start_of_next_chunk() {
        let mut event_log = InMemoryEventLog::new();
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        });
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 10,
            payload: EventPayload::MetadataUpdated {
                project_id: "proj-123".to_string(),
                meta_ptr: MetaPtr {
                    pointer: "123".to_string(),
                },
            },
        });
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 20,
            payload: EventPayload::OwnerAdded {
                project_id: "proj-123".to_string(),
                owner: "0x123".to_string(),
            },
        });

        let (events, next_event_index) = event_log.range(1, 2);

        assert_eq!(next_event_index, 3);
        assert_eq!(events[0].block_number, 10);
        assert_eq!(events[1].block_number, 20);
    }
}
