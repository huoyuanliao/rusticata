use nom::IResult;
use nom::HexDisplay;
use openvpn_parser::{parse_openvpn_udp,Payload};
use tls::TlsParser;

use rparser::{RParser,R_STATUS_OK,R_STATUS_FAIL};

pub struct OpenVPNUDPParser<'a> {
    _name: Option<&'a[u8]>,
    defrag_buf: Vec<u8>,

    tls_parser: TlsParser<'a>,
}

impl<'a> OpenVPNUDPParser<'a> {
    pub fn new(name: &'a[u8]) -> OpenVPNUDPParser<'a> {
        OpenVPNUDPParser{
            _name: Some(name),
            defrag_buf: Vec::new(),
            tls_parser: TlsParser::new(b"OpenVPN/TLS"),
        }
    }
}


impl<'a> RParser for OpenVPNUDPParser<'a> {
    fn parse(&mut self, i: &[u8], _direction: u8) -> u32 {
        let mut cur_i = i;
        loop {
            match parse_openvpn_udp(cur_i) {
                IResult::Done(rem,r) => {
                    debug!("parse_openvpn_udp: {:?}", r);
                    if let Payload::Control(ref ctrl) = r.msg {
                        // XXX check number, and if packets needs to be reordered
                        self.defrag_buf.extend_from_slice(ctrl.payload);
                    }
                    if rem.len() == 0 { break; }
                    debug!("Remaining bytes: {}", rem.len());
                    cur_i = rem;
                },
                e @ _ => {
                    warn!("parse_openvpn_udp failed: {:?}", e);
                    warn!("input buffer:\n{}",i.to_hex(16));
                    return R_STATUS_FAIL;
                },
            }
        }
        if self.defrag_buf.len() > 0 {
            // inscpect TLS message
            debug!("TLS message:\n{}", self.defrag_buf.to_hex(16));
            self.tls_parser.parse_tcp_level(&self.defrag_buf);
            self.defrag_buf.clear();
        }
        R_STATUS_OK
    }
}

pub fn openvpn_udp_probe(i: &[u8]) -> bool {
    if i.len() <= 20 { return false; }
    // XXX
    if let IResult::Done(_,_) = parse_openvpn_udp(i) { return true; }
    return false;
}

