#[macro_export]
macro_rules! jobs {
    (
        $($id:ident,)*
    ) => {
        $(
            pub struct $id;

            impl Job for $id {
                #[inline]
                fn name(&self) -> JobName {
                    JobName::from(stringify!($id))
                }

                #[inline]
                fn perform(&self, args: &Args, con: &WorkerConnection) -> JobResult {
                    $id::perform(args.deserialize()?, con)
                }
            }

            impl $id {
                #[allow(dead_code)]
                #[inline]
                pub fn perform_now<A: Serialize>(
                    args: A,
                    con: &WorkerConnection,
                ) -> RobinResult<()> {
                    $id.perform_now(args, con)
                }

                #[allow(dead_code)]
                #[inline]
                pub fn perform_later<A: Serialize>(
                    args: A,
                    con: &WorkerConnection,
                ) -> RobinResult<()> {
                    $id.perform_later(args, con)
                }
            }
        )*

        pub fn __robin_lookup_job(name: &JobName) -> Option<Box<Job + Send>> {
            match name.0.as_ref() {
                $(
                    stringify!($id) => Some(Box::new($id)),
                )*
                _ => None,
            }
        }
    }
}

#[macro_export]
macro_rules! robin_establish_connection {
    ($config:expr) => (
        robin::connection::establish($config.clone(), __robin_lookup_job)
    )
}

#[macro_export]
macro_rules! robin_boot_worker {
    ($config:expr) => (
        robin::worker::boot(&$config.clone(), __robin_lookup_job);
    )
}
