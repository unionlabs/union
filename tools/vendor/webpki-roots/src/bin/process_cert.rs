extern crate webpki;
use std::io;
use std::io::Read;

fn dumphex(label: &str, bytes: &[u8]) {
  print!("{}: ", label);
  for b in bytes {
    print!("{:02x}", b);
  }
  println!("");
}

fn main() {
  let mut der = Vec::new();
  io::stdin().read_to_end(&mut der)
    .expect("cannot read stdin");

  let ta = webpki::TrustAnchor::try_from_cert_der(&der)
    .expect("cannot parse certificate");

  dumphex("Subject", ta.subject);
  dumphex("SPKI", ta.spki);
  if ta.name_constraints.is_none() {
    println!("Name-Constraints: None");
  } else {
    dumphex("Name-Constraints", ta.name_constraints.unwrap());
  }
}
