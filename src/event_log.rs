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
        let actual_end =
            if desired_end > self.events.len() {
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
    use crate::event_handling::EventPayload;

    use super::*;

    #[test]
    fn test_handle_empty_log() {
        let event_log = InMemoryEventLog { events: Vec::new() };

        let start = 0;
        let (_, next_event_index) = event_log.range(start, 2);

        assert_eq!(next_event_index, 0);
    }

    #[test]
    fn test_handle_exhausting_log() {
        let mut event_log = InMemoryEventLog { events: Vec::new() };
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        });

        let (_, next_event_index_1) = event_log.range(0, 2);
        let (_, next_event_index_2) = event_log.range(next_event_index_1, 2);
        let (_, next_event_index_3) = event_log.range(next_event_index_2, 2);

        assert_eq!(next_event_index_3, 1);
    }
}
