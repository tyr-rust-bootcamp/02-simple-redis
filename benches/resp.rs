use anyhow::Result;
use bytes::BytesMut;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_redis::{parse_frame, parse_frame_length, RespFrame};

// resp frames covers all kinds of real-world redis requests and responses
// cmd 1: set key value
// cmd 1 response: OK
// cmd 2: get key
// cmd 2 response: value
// cmd 3: hset key field value
// cmd 3 response: ERR
// cmd 4: hget key field
// cmd 4 response: value
// cmd 5: sadd key member
// cmd 5 response: 1
const DATA: &str = "*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*1\r\n+OK\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*4\r\n$4\r\nHSET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*1\r\n-ERR\r\n*3\r\n$4\r\nHGET\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n*3\r\n$4\r\nSADD\r\n$3\r\nkey\r\n$6\r\nmember\r\n:1\r\n";

fn v1_decode(buf: &mut BytesMut) -> Result<Vec<RespFrame>> {
    use simple_redis::RespDecode;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode(buf: &mut BytesMut) -> Result<Vec<RespFrame>> {
    use simple_redis::RespDecodeV2;
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = RespFrame::decode(buf)?;
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode_no_buf_clone(buf: &mut &[u8]) -> Result<Vec<RespFrame>> {
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let _len = parse_frame_length(buf)?;

        let frame = parse_frame(buf).unwrap();
        frames.push(frame);
    }
    Ok(frames)
}

fn v2_decode_parse_length(buf: &mut &[u8]) -> Result<()> {
    use simple_redis::RespDecodeV2;
    while !buf.is_empty() {
        let len = RespFrame::expect_length(buf)?;
        *buf = &buf[len..];
    }
    Ok(())
}

fn v1_decode_parse_length(buf: &mut &[u8]) -> Result<()> {
    use simple_redis::RespDecode;
    while !buf.is_empty() {
        let len = RespFrame::expect_length(buf)?;
        *buf = &buf[len..];
    }
    Ok(())
}

fn v2_decode_parse_frame(buf: &mut &[u8]) -> Result<Vec<RespFrame>> {
    let mut frames = Vec::new();
    while !buf.is_empty() {
        let frame = parse_frame(buf).unwrap();
        frames.push(frame);
    }
    Ok(frames)
}

fn criterion_benchmark(c: &mut Criterion) {
    let buf = BytesMut::from(DATA);

    c.bench_function("v1_decode", |b| {
        b.iter(|| v1_decode(black_box(&mut buf.clone())))
    });

    c.bench_function("v2_decode", |b| {
        b.iter(|| v2_decode(black_box(&mut buf.clone())))
    });

    c.bench_function("v2_decode_no_buf_clone", |b| {
        b.iter(|| v2_decode_no_buf_clone(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("v1_decode_parse_length", |b| {
        b.iter(|| v1_decode_parse_length(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("v2_decode_parse_length", |b| {
        b.iter(|| v2_decode_parse_length(black_box(&mut DATA.as_bytes())))
    });

    c.bench_function("v2_decode_parse_frame", |b| {
        b.iter(|| v2_decode_parse_frame(black_box(&mut DATA.as_bytes())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
