#[macro_export]
macro_rules! jobs {
    (
        $($id:ident,)*
    ) => {
        #[derive(Job)]
        enum __RobinJobs {
            $(
                $id,
            )*
        }

        $(
            pub struct $id;

            impl Job for $id {
                #[inline]
                fn name(&self) -> JobName {
                    __RobinJobs::$id.name()
                }

                #[inline]
                fn perform(&self, args: &Args, con: &WorkerConnection) -> JobResult {
                    __RobinJobs::$id.perform(args, con)
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
    }
}

#[macro_export]
macro_rules! robin_establish_connection {
    ($config:expr) => (
        robin::connection::establish($config.clone(), jobs::lookup_job)
    )
}

#[macro_export]
macro_rules! robin_boot_worker {
    ($config:expr) => (
        robin::worker::boot(&$config.clone(), jobs::lookup_job);
    )
}
