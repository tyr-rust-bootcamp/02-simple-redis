use anyhow::Result;
use bytes::BytesMut;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_redis::{parse_frame, parse_frame_length, RespFrame};

const DATA: &str = "+OK\r\n-ERR\r\n:1000\r\n$6\r\nfoobar\r\n$-1\r\n*2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n";

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
