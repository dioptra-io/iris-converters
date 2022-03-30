use std::collections::HashMap;
use chrono::Utc;
use sha2::{Digest, Sha256};
use crate::{AtlasIcmpExt, AtlasIcmpExtMplsData, AtlasIcmpExtObj, AtlasTraceroute, AtlasTracerouteHop, AtlasTracerouteReply, IrisReply, IrisTraceroute};



fn id_from_string(s: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(s);
    let result = hasher.finalize();
    u64::from_le_bytes(result.as_slice()[..8].try_into().unwrap())
}

impl IrisTraceroute {
    pub fn to_atlas_traceroute(&self, measurement_uuid: &str, agent_uuid: &str) -> AtlasTraceroute {
        let protocols = HashMap::from([
            (1, "icmp"),
            (17, "udp"),
            (58, "icmp6"),
        ]);
        let start_timestamp = self.replies.iter().map(|reply| reply.0).min().unwrap();
        let end_timestamp = self.replies.iter().map(|reply| reply.0).max().unwrap();
        AtlasTraceroute {
            af: self.af(),
            dst_addr: self.probe_dst_addr,
            dst_name: self.probe_dst_addr.to_string(),
            endtime: end_timestamp,
            from: self.probe_src_addr,
            msm_id: id_from_string(measurement_uuid),
            msm_name: String::from(measurement_uuid),
            paris_id: self.probe_src_port,
            prb_id: id_from_string(agent_uuid),
            proto: protocols[&self.probe_protocol].parse().unwrap(),
            result: self.replies.iter().map(|reply| reply.to_atlas_hop()).collect(),
            size: 0, // TODO
            src_addr: self.probe_src_addr,
            timestamp: start_timestamp,
            kind: "traceroute".to_string(),
        }
    }
}

impl IrisReply {
    pub fn to_atlas_hop(&self) -> AtlasTracerouteHop {
        AtlasTracerouteHop { hop: self.1, result: vec![self.to_atlas_reply()] }
    }

    pub fn to_atlas_reply(&self) -> AtlasTracerouteReply {
        let mut icmpext = vec![];
        if !self.4.is_empty() {
            let mpls = self.4.iter().map(|entry| {
                AtlasIcmpExtMplsData {
                    label: entry.0,
                    exp: entry.1,
                    s: entry.2,
                    ttl: entry.3,
                }
            }).collect();
            let obj = AtlasIcmpExtObj {
                class: 1,
                kind: 1,
                mpls,
            };
            let ext = AtlasIcmpExt {
                version: 2,
                rfc4884: 1,
                obj: vec![obj],
            };
            icmpext.push(ext);
        }
        AtlasTracerouteReply {
            from: self.5,
            rtt: self.6,
            size: self.3,
            ttl: self.2,
            icmpext,
        }
    }
}