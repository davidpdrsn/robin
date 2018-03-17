pub mod redis_queue;

#[derive(Deserialize, Serialize)]
struct EnqueuedJob {
    name: String,
    args: String,
}
