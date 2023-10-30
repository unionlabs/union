
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

const REQ_SHORT: &[u8] = b"\
GET / HTTP/1.0\r\n\
Host: example.com\r\n\
Cookie: session=60; user_id=1\r\n\r\n";

const REQ: &[u8] = b"\
GET /wp-content/uploads/2010/03/hello-kitty-darth-vader-pink.jpg HTTP/1.1\r\n\
Host: www.kittyhell.com\r\n\
User-Agent: Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: ja,en-us;q=0.7,en;q=0.3\r\n\
Accept-Encoding: gzip,deflate\r\n\
Accept-Charset: Shift_JIS,utf-8;q=0.7,*;q=0.7\r\n\
Keep-Alive: 115\r\n\
Connection: keep-alive\r\n\
Cookie: wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256\r\n\r\n";

fn req(c: &mut Criterion) {
    let mut headers = [httparse::Header{ name: "", value: &[] }; 16];
    let mut req = httparse::Request::new(&mut headers);

    c.benchmark_group("req")
        .throughput(Throughput::Bytes(REQ.len() as u64))
        .bench_function("req", |b| b.iter(|| {
            assert_eq!(black_box(req.parse(REQ).unwrap()), httparse::Status::Complete(REQ.len()));
        }));
}

fn req_short(c: &mut Criterion) {
    let mut headers = [httparse::Header{ name: "", value: &[] }; 16];
    let mut req = httparse::Request::new(&mut headers);

    c.benchmark_group("req_short")
        .throughput(Throughput::Bytes(REQ_SHORT.len() as u64))
        .bench_function("req_short", |b| b.iter(|| {
            assert_eq!(black_box(req.parse(REQ_SHORT).unwrap()), httparse::Status::Complete(REQ_SHORT.len()));
        }));
}

const RESP_SHORT: &[u8] = b"\
HTTP/1.0 200 OK\r\n\
Date: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
Set-Cookie: session=60; user_id=1\r\n\r\n";

// These particular headers don't all make semantic sense for a response, but they're syntactically valid.
const RESP: &[u8] = b"\
HTTP/1.1 200 OK\r\n\
Date: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
Host: www.kittyhell.com\r\n\
User-Agent: Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: ja,en-us;q=0.7,en;q=0.3\r\n\
Accept-Encoding: gzip,deflate\r\n\
Accept-Charset: Shift_JIS,utf-8;q=0.7,*;q=0.7\r\n\
Keep-Alive: 115\r\n\
Connection: keep-alive\r\n\
Cookie: wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256\r\n\r\n";

fn resp(c: &mut Criterion) {
    let mut headers = [httparse::Header{ name: "", value: &[] }; 16];
    let mut resp = httparse::Response::new(&mut headers);

    c.benchmark_group("resp")
        .throughput(Throughput::Bytes(RESP.len() as u64))
        .bench_function("resp", |b| b.iter(|| {
            assert_eq!(black_box(resp.parse(RESP).unwrap()), httparse::Status::Complete(RESP.len()));
        }));
}

fn resp_short(c: &mut Criterion) {
    let mut headers = [httparse::Header{ name: "", value: &[] }; 16];
    let mut resp = httparse::Response::new(&mut headers);

    c.benchmark_group("resp_short")
        .throughput(Throughput::Bytes(RESP_SHORT.len() as u64))
        .bench_function("resp_short", |b| b.iter(|| {
            assert_eq!(black_box(resp.parse(RESP_SHORT).unwrap()), httparse::Status::Complete(RESP_SHORT.len()));
        }));
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(100).measurement_time(Duration::from_secs(10));
    targets = req, req_short, resp, resp_short
}
criterion_main!(benches);
