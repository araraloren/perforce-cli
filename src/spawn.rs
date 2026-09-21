use std::process::Child;
use std::process::Output;

pub trait ParameterizedSpawn {
    type Input<'a>;
    type Output<'a>;
    type Error;

    fn spawn_with<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error>;
}

pub trait SpawnExt: ParameterizedSpawn {
    fn spawn<'a>(&mut self) -> Result<Self::Output<'a>, Self::Error>;
}

impl<T> SpawnExt for T
where
    T: for<'a> ParameterizedSpawn<Input<'a> = ()>,
{
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

/// Generates `SpawnExtN<T1, …, TN>` + `OutputExtN<T1, …, TN>` traits and
/// their blanket impls for a given arity N ≥ 2.
///
/// For N args, the `ParameterizedSpawn::Input<'a>` must be `(T1, …, TN)`.
/// The generated `spawn` method takes N separate parameters and passes them
/// as a tuple to `spawn_with`; `output` does the same via `output_with`.
macro_rules! spawnext_def {
    (
        $spawn_trait:ident,
        $output_trait:ident,
        $($param:ident : $T:ident),+
    ) => {
        #[allow(clippy::too_many_arguments)]
        pub trait $spawn_trait<$($T),+>: ParameterizedSpawn {
            fn spawn<'a>(
                &mut self,
                $($param: $T),+
            ) -> Result<Self::Output<'a>, Self::Error>;
        }

        #[allow(clippy::too_many_arguments)]
        #[allow(unused_parens)]
        impl<T, $($T),+> $spawn_trait<$($T),+> for T
        where
            T: for<'a> ParameterizedSpawn<Input<'a> = ($($T),+)>,
        {
            fn spawn<'a>(
                &mut self,
                $($param: $T),+
            ) -> Result<Self::Output<'a>, Self::Error> {
                self.spawn_with(($($param),+))
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub trait $output_trait<$($T),+>: ParameterizedOutput {
            fn output<'a>(
                &mut self,
                $($param: $T),+
            ) -> Result<Output, Self::Error>
            where
                $($T: 'a),+;
        }

        #[allow(clippy::too_many_arguments)]
        #[allow(unused_parens)]
        impl<T, $($T),+> $output_trait<$($T),+> for T
        where
            T: for<'a> ParameterizedOutput<Input<'a> = ($($T),+)>,
        {
            fn output<'a>(
                &mut self,
                $($param: $T),+
            ) -> Result<Output, Self::Error>
            where
                $($T: 'a),+,
            {
                self.output_with(($($param),+))
            }
        }
    };
}

spawnext_def!(SpawnExt1, OutputExt1, a: T1);

spawnext_def!(SpawnExt2, OutputExt2, a: T1, b: T2);
spawnext_def!(SpawnExt3, OutputExt3, a: T1, b: T2, c: T3);
spawnext_def!(SpawnExt4, OutputExt4, a: T1, b: T2, c: T3, d: T4);
spawnext_def!(SpawnExt5, OutputExt5, a: T1, b: T2, c: T3, d: T4, e: T5);
spawnext_def!(SpawnExt6, OutputExt6, a: T1, b: T2, c: T3, d: T4, e: T5, f: T6);
spawnext_def!(SpawnExt7, OutputExt7, a: T1, b: T2, c: T3, d: T4, e: T5, f: T6, g: T7);
spawnext_def!(SpawnExt8, OutputExt8, a: T1, b: T2, c: T3, d: T4, e: T5, f: T6, g: T7, h: T8);
