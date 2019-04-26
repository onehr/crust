extern crate crust;
#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;

use crust::cpp;
use std::{error, fs, path::PathBuf};

fn criterion_benchmark(c: &mut Criterion) -> Result<(), Box<dyn error::Error>> {
    // TODO: change this to iter through every c file under "test/valid/cpp"
    let input_files = &[
        "test/valid/cpp/macro_object.c",
        "test/valid/cpp/macro_object_2.c",
        "test/valid/cpp/trash.c",
        "test/valid/cpp/header1.c",
        "test/valid/cpp/comment_1.c",
        "test/valid/cpp/tri_1.c",
        "test/valid/cpp/tri_2.c",
        "test/valid/cpp/tri_3.c",
    ];
    for input_file in input_files.iter() {
        c.bench_function(&format!("cpp {}", input_file), move |b| {
            b.iter(|| {
                cpp::cpp_driver(
                    black_box(fs::read_to_string(input_file).unwrap()),
                    PathBuf::from(input_file),
                )
            })
        });
    }
    return Ok(());
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
