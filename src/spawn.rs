use std::process::Child;
use std::process::Output;

pub trait ParameterizedSpawn<T> {
    type Output;
    type Error;

    fn spawn_with(&mut self, input: T) -> Result<Self::Output, Self::Error>;
}

pub trait ParameterizedOutput<T> {
    type Error;

    fn output_with(&mut self, input: T) -> Result<Output, Self::Error>;
}

impl<T, I> ParameterizedOutput<I> for T
where
    T: ParameterizedSpawn<I, Output = Child, Error = std::io::Error>,
{
    type Error = std::io::Error;

    fn output_with(&mut self, input: I) -> Result<Output, Self::Error> {
        self.spawn_with(input)
            .and_then(|child| child.wait_with_output())
    }
}

pub trait SpawnExt: ParameterizedSpawn<()> {
    fn spawn(&mut self) -> Result<Self::Output, Self::Error>;
}

impl<T> SpawnExt for T
where
    T: ParameterizedSpawn<()>,
{
    fn spawn(&mut self) -> Result<Self::Output, Self::Error> {
        self.spawn_with(())
    }
}

pub trait OutputExt: SpawnExt {
    fn output(&mut self) -> Result<Output, Self::Error>;
}

impl<T> OutputExt for T
where
    T: SpawnExt<Output = Child, Error = std::io::Error>,
{
    fn output(&mut self) -> Result<Output, Self::Error> {
        self.spawn().and_then(|child| child.wait_with_output())
    }
}

/// Generates `SpawnExtN<T1, …, TN>` + `OutputExtN<T1, …, TN>` traits and
/// their blanket impls for a given arity N ≥ 1.
///
/// For N args, the input type must be `(T1, …, TN)`. The generated `spawn`
/// method takes N separate parameters and passes them as a tuple to
/// `spawn_with`; `output` does the same via `output_with`.
macro_rules! spawnext_def {
    (
        $spawn_trait:ident,
        $output_trait:ident,
        $($param:ident : $T:ident),+
    ) => {
        #[allow(clippy::too_many_arguments)]
        #[allow(unused_parens)]
        pub trait $spawn_trait<$($T),+>: ParameterizedSpawn<($($T,)+)> {
            fn spawn(
                &mut self,
                $($param: $T),+
            ) -> Result<Self::Output, Self::Error>;
        }

        #[allow(clippy::too_many_arguments)]
        #[allow(unused_parens)]
        impl<T, $($T),+> $spawn_trait<$($T),+> for T
        where
            T: ParameterizedSpawn<($($T,)+)>,
        {
            fn spawn(
                &mut self,
                $($param: $T),+
            ) -> Result<Self::Output, Self::Error> {
                self.spawn_with(($($param,)+))
            }
        }

        #[allow(clippy::too_many_arguments)]
        #[allow(unused_parens)]
        pub trait $output_trait<$($T),+>: $spawn_trait<$($T),+> {
            fn output(
                &mut self,
                $($param: $T),+
            ) -> Result<Output, Self::Error>;
        }

        #[allow(clippy::too_many_arguments)]
        #[allow(unused_parens)]
        impl<T, $($T),+> $output_trait<$($T),+> for T
        where
            T: $spawn_trait<$($T),+, Output = std::process::Child, Error = std::io::Error>,
        {
            fn output(
                &mut self,
                $($param: $T),+
            ) -> Result<Output, Self::Error> {
                self.spawn($($param),+).and_then(|child| child.wait_with_output())
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
