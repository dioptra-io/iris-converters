use std::ops::Add;

use chrono::{Duration, TimeZone, Utc};
use warts::{Timeval, TraceProbe, TraceType};

use crate::formats::internal::{Protocol, Traceroute, TracerouteFlow, TracerouteReply};
use crate::formats::scamper_trace_warts::models::ScamperTraceWarts;
use crate::utils::UNSPECIFIED;

impl From<&ScamperTraceWarts> for Traceroute {
    fn from(meta: &ScamperTraceWarts) -> Self {
        Traceroute {
            measurement_id: meta.cycle_id.to_string(),
            agent_id: meta.monitor_name.to_string(),
            start_time: Default::default(), // TODO
            // start_time: Utc
            //     .timestamp_opt(traceroute.start_time.map_or(|t| t.seconds, 0) as i64, 0)
            //     .unwrap(),
            end_time: Default::default(), // TODO
            protocol: meta.traceroute.trace_type.as_ref().unwrap().into(),
            src_addr: meta
                .traceroute
                .src_addr
                .map(|x| x.into())
                .unwrap_or(UNSPECIFIED),
            dst_addr: meta
                .traceroute
                .dst_addr
                .map(|x| x.into())
                .unwrap_or(UNSPECIFIED),
            flows: vec![TracerouteFlow {
                src_port: meta.traceroute.src_port.unwrap_or(0),
                dst_port: meta.traceroute.dst_port.unwrap_or(0),
                replies: meta.traceroute.hops.iter().map(|tp| tp.into()).collect(),
            }],
        }
    }
}

impl From<&TraceProbe> for TracerouteReply {
    fn from(tp: &TraceProbe) -> Self {
        let tx = tp.tx.as_ref().unwrap_or(&Timeval {
            seconds: 0,
            microseconds: 0,
        });
        let capture_timestamp = Utc
            .timestamp_opt(tx.seconds as i64, tx.microseconds * 1000)
            .unwrap()
            .add(Duration::microseconds(tp.rtt_usec.unwrap_or(0) as i64));
        TracerouteReply {
            timestamp: capture_timestamp,
            probe_ttl: tp.probe_ttl.unwrap_or(0),
            quoted_ttl: tp.quoted_ttl.unwrap_or(0),
            ttl: tp.reply_ttl.unwrap_or(0),
            size: tp.reply_size.unwrap_or(0),
            mpls_labels: vec![], // TODO
            addr: tp.addr.map(|x| x.into()).unwrap_or(UNSPECIFIED),
            icmp_type: tp.icmp_type.unwrap_or(0),
            icmp_code: tp.icmp_code.unwrap_or(0),
            rtt: tp.rtt_usec.unwrap_or(0) as f64 / 1000.0,
        }
    }
}

impl From<&TraceType> for Protocol {
    // TODO: IPv6
    fn from(value: &TraceType) -> Self {
        match value {
            TraceType::ICMPEcho => Protocol::ICMP,
            TraceType::UDP => Protocol::UDP,
            TraceType::TCP => Protocol::TCP,
            TraceType::ICMPEchoParis => Protocol::ICMP,
            TraceType::UDPParis => Protocol::UDP,
            TraceType::TCPAck => Protocol::TCP,
        }
    }
}
