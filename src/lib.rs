/// the etc/hosts file is used to statically define local dns records
/// the format of this file is quite simple
///
/// address \t name, name.domain, name-alias, name-aliai
/// address \t name
///
/// or any combination of the sort
use std::net::IpAddr;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("Invalid Ipv4Addr: Must be private or global")]
    InvalidIpAddress(String),
}

/// Record is a way of representing a single entry in the hosts files
#[derive(Debug, Eq, PartialEq)]
pub struct Record {
    /// addr for the record
    addr: IpAddr,
    /// here we have multiple names for a single record
    names: Vec<String>,
}
impl Record {
    pub fn new(addr: IpAddr, names: Vec<String>) -> Result<Self, RecordError> {
        // I would love to use is_global here as well but it is only a nightly feature
        // may upgrade to nightly later on
        if addr.is_ipv4() || addr.is_ipv4() {
            return Ok(Self {
                addr: addr,
                names: names,
            });
        }

        Err(RecordError::InvalidIpAddress(addr.to_string()))
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    CouldNotOpen(#[from] std::io::Error),

    #[error("bad ipv4 addr, not global or loopback")]
    ParseError(#[from] std::net::AddrParseError),

    #[error("unknown")]
    Unknown(String),
}

#[derive(Debug, Default)]
pub enum Part {
    Addr,
    Names,
    Comment,
    #[default]
    Unknown,
}

/// Parser is a way we can extract Records from the etc/hosts file
#[derive(Debug)]
struct Parser {
    line: i64,
    part: Part,
    records: Vec<Record>
}

impl Default for Parser {
    fn default() -> Parser {
        let records: Vec<Record> = Vec::new();
        Parser {line: 0, part: Part::Unknown, records: records}
    }
}

impl Parser {
    pub fn parse(&mut self, file: &Path) -> Result<Vec<Record>, ParserError> {
        let file = File::open(file)?;
        let buff = io::BufReader::new(file).lines();

        self.part = Part::Names;

        for line in buff {
            if let Ok(a) = line {
                if a.is_empty() { continue; }
                if a.starts_with('#') { continue; }
                // dont worry about tabs, gersh darnit
                let a = a.replace("\t", " ");

                let mut record_info =
                    a.split(' ').filter(|&s| !s.is_empty()).collect::<Vec<&str>>();

                let name = record_info.remove(0).to_string();

                let addrs =
                    record_info.iter().map(|&s| s.to_string()).collect::<Vec<String>>();

                if let Ok(record) = Record::new(name.parse()?, addrs) {
                    self.records.push(record);
                };
            }
        }

        Ok(self.records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_loopback() {
        let addr = "127.0.0.1".parse().unwrap();
        let names: Vec<String> = vec!["localhost".to_string()];
        let record = Record::new(addr, names);
        assert!(record.is_ok())
    }

    #[test]
    fn create_private() {
        let addr = "192.168.10.42".parse().unwrap();
        let names: Vec<String> = vec!["core.naus".to_string()];
        let record = Record::new(addr, names);
        assert!(record.is_ok())
    }

    #[test]
    fn test_parser() {
        use std::path::Path;
        let mut parser: Parser = Default::default();
        let path = Path::new("/etc/hosts");
        match parser.parse(path) {
            Ok(_v) => println!("good to go"),
            Err(e) => println!("{e:?}"),
        }
    }
}
