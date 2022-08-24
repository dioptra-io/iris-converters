use crate::internal::TracerouteReply;
use crate::warts_trace::to::warts_trace_to_internal;
use std::io::Read;
use warts::{Address, Object, Traceroute};

pub struct WartsTraceReader {
    cycle_id: u32,
    monitor_name: String,
    traceroutes: Vec<Traceroute>,
}

impl WartsTraceReader {
    pub fn new<R: Read>(mut input: R) -> WartsTraceReader {
        let mut reader = WartsTraceReader {
            cycle_id: 0,
            monitor_name: "unknown".to_string(),
            traceroutes: Vec::new(),
        };

        // We currently read warts file in a single batch:
        // https://github.com/sharksforarms/deku/issues/105
        // Once this is implemented, we could move this logic to next().
        let mut data: Vec<u8> = Vec::new();
        input.read_to_end(&mut data).unwrap();
        let objects = Object::all_from_bytes(&data);

        // We assume that a warts file contains one and only one cycle; find the first one.
        for object in &objects {
            match object {
                Object::CycleDefinition(cycle_start) | Object::CycleStart(cycle_start) => {
                    let hostname = cycle_start.hostname.as_ref().unwrap().clone();
                    reader.cycle_id = cycle_start.cycle_id;
                    reader.monitor_name = hostname.into_string().unwrap();
                    break;
                }
                _ => {}
            }
        }

        // Old warts files, such as the ones produced by Ark around 2007-2010, use a global address
        // table, instead of an address table per traceroute as is the case in newer files.
        // As such, all the objects must be read before de-referencing the addresses contained in
        // the traceroutes. We could get rid of this by removing support for these old files.
        let mut table = Vec::new();
        for object in &objects {
            if let Object::Address(address) = object {
                table.push(Address::from(*address))
            }
        }

        // Dereference addresses and store the update traceroute object.
        for mut object in objects {
            if table.is_empty() {
                // Newer format, the address table is local to the traceroute.
                object.dereference();
            } else {
                // Older format, use the global address table.
                object.dereference_with_table(&table);
            }
            if let Object::Traceroute(traceroute) = object {
                reader.traceroutes.push(traceroute);
            }
        }
        reader
    }
}

impl Iterator for WartsTraceReader {
    type Item = anyhow::Result<Vec<TracerouteReply>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.traceroutes.pop() {
            Some(traceroute) => {
                let internal =
                    warts_trace_to_internal(&traceroute, self.cycle_id, &self.monitor_name);
                Some(Ok(internal))
            }
            _ => None,
        }
    }
}
