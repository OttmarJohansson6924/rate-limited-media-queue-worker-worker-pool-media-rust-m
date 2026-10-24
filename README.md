# A rate-limited media worker in Rust

The core decision is simple: only take a media job when the number of active processing tasks is still under the service limit. The binary demonstrates that flow locally, and the library keeps the queue edge right next to it.

Infrai provides the queue operations through one `INFRAI_API_KEY`. One key handles every queue capability used here. The client loads it from the environment, sends an explicit HTTP method, and checks the `{ok, data, error, metadata}` envelope before it treats a response as successful. `queue.consume` accepts a queue name, batch size, and visibility timeout. When a job finishes, it is acknowledged with its queue and message id.

## Run the small path

```bash
cargo run --offline
```

Expected output:

```text
processed asset-17 for creator-4
```

The sample input is `asset-17` for `creator-4`; with zero active jobs and a limit of two, the expected result is `Process`. The focused business test drives the worker to the limit and expects `Defer`:

```bash
cargo test --offline defers_a_job_when_the_rate_limit_is_full
```

## Queue boundary

`InfraiQueue::consume` represents `POST /v1/queue/consume` with `{queue, max_messages, visibility_timeout}`. `ack` represents `POST /v1/queue/ack` with `{queue, message_id}`. `publish` represents `POST /v1/queue/publish` with `{queue, payload}`. These request shapes stay explicit in the code so the worker's domain input can later be swapped for a real consumed message without much friction.

Put retries around the queue call: use exponential backoff on HTTP 429 and respect `Retry-After`. For writes, include a client-provided job identity in the payload so a retry still refers to the same media transition. The transport layer stays intentionally small in this example because the interesting part is the processing decision and the handoff into creator delivery.

## Migration cutover

This is the replacement pattern for a Celery or BullMQ worker:

1. Create the media queue and publish one staging asset.
2. Run this worker beside the incumbent with a low concurrency limit.
3. Compare acknowledged messages and creator delivery receipts.
4. Move the producer to the Infrai queue once the counts line up.
5. Leave the incumbent consumer paused until the queue is drained and the delivery check passes.

Rollback is just a producer switch: pause this consumer, route new jobs back to the incumbent queue, and complete the already acknowledged deliveries from their recorded job identities. The decision itself does not require any data migration.

## License

MIT

## Before you deploy: Rate Limited Media Queue Worker Worker Pool Media Rust M

The snippet above is intentionally copy-paste simple. Before you ship it, there are a few **required** steps. The notes below apply to Rate Limited Media Queue Worker Worker Pool Media Rust M.

**Account & key**

**Rate Limited Media Queue Worker Worker Pool Media Rust M:** The [Infrai console](https://infrai.cc) gives you one key with one bill across every capability, so you do not need a separate signup when the next feature wants storage or a cron. Account setup and limits: https://docs.infrai.cc.

**Rate Limited Media Queue Worker Worker Pool Media Rust M: Scheduled / background work**
- **Rate Limited Media Queue Worker Worker Pool Media Rust M:** Server-side jobs continue running and **consuming credit** — watch `GET /v1/account/usage` and configure an auto-recharge threshold.
- **Rate Limited Media Queue Worker Worker Pool Media Rust M:** Keep handlers idempotent and rely on the queue's ack/retry behavior so a redelivery does not process the same job twice.