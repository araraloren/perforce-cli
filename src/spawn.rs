use std::process::Child;
use std::process::Output;

pub trait ParameterizedSpawn {
    type Input<'a>;
    type Output<'a>;
    type Error;

    fn spawn_with<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error>;
}

pub trait SpawnExt {
    type Output<'a>;
    type Error;

    fn spawn<'a>(&mut self) -> Result<Self::Output<'a>, Self::Error>;
}

impl<T> SpawnExt for T
where
    T: for<'a> ParameterizedSpawn<Input<'a> = ()>,
{
    type Output<'a> = T::Output<'a>;
    type Error = T::Error;

    fn spawn<'a>(&mut self) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_with(())
    }
}

pub trait ParameterizedOutput {
    type Input<'a>;
    type Error;

    fn output_with<'a>(&mut self, input: Self::Input<'a>) -> Result<Output, Self::Error>;
}

impl<T> ParameterizedOutput for T
where
    T: for<'a> ParameterizedSpawn<Output<'a> = Child, Error = std::io::Error>,
{
    type Input<'a> = T::Input<'a>;
    type Error = std::io::Error;

    fn output_with<'a>(&mut self, input: Self::Input<'a>) -> Result<Output, Self::Error> {
        self.spawn_with(input)
            .and_then(|child| child.wait_with_output())
    }
}

pub trait OutputExt: SpawnExt {
    fn output(&mut self) -> Result<Output, Self::Error>;
}

impl<T> OutputExt for T
where
    T: for<'a> SpawnExt<Output<'a> = Child, Error = std::io::Error>,
{
    fn output(&mut self) -> Result<Output, Self::Error> {
        self.spawn().and_then(|child| child.wait_with_output())
    }
}
