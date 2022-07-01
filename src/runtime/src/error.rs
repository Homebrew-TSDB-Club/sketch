use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum RuntimeError {
    #[snafu(display("get host cores info error"))]
    GetCoreError,
    #[snafu(display("not too much cores, require: {}, has: {}", require, has))]
    NotMuchCores { require: usize, has: usize },
}
