# Hosting Tintype on Cloud Run

This guide reproduces the hosted deployment: a token-protected API and the
public demo behind the [website's "Try it" section](https://cheesejaguar.github.io/tintype/#try),
as two Cloud Run services built from one image. It contains no project IDs,
URLs of the private API or secrets; replace the shell variables with your own.

The bundled model weights are licensed CC BY-NC-SA 4.0, so a hosted service
built from them must stay free and non-commercial.

## Why Cloud Run, and when to move

- Cloud Run scales to zero, so an idle service costs nothing. Its monthly free
  allowance covers roughly 130,000 to 300,000 photos, depending on photo size.
  After that, expect roughly $15 to $35 per million photos.
- At around a million photos a month, a fixed ARM VM running
  `tintype serve --turbo` costs less.
- Cloud Run is x86-64, and you don't choose the CPU model. Turbo mode is only
  faster on CPUs with VNNI. Start in standard mode and check before turning
  turbo on ([Turbo on Cloud Run](#turbo-on-cloud-run)).

## Region

Use `us-central1` unless the
[Cloud Run price list](https://cloud.google.com/run/pricing) shows a cheaper US
region.

On 2026-11-16, every US region in Tier 1 cost the same: us-central1, us-east1,
us-east4, us-east5, us-south1, us-west1 and us-west8. Tier 2 (us-west2, us-west3
and us-west4) costs more. Keep the Artifact Registry repository in the same
region, so image pulls are free.

```sh
PROJECT=your-project-id
REGION=us-central1
SITE_ORIGIN=https://cheesejaguar.github.io   # the website that embeds the demo
IMAGE=$REGION-docker.pkg.dev/$PROJECT/tintype/tintype:$(git rev-parse --short HEAD)
gcloud config set project "$PROJECT"
```

## 1. Enable services and build the image

The published container images can be older than the server's demo mode, so
build the image from the source you are deploying.

[`deploy/cloudbuild.yaml`](../deploy/cloudbuild.yaml) builds the Dockerfile's
`bundled` target for x86-64. [`.gcloudignore`](../.gcloudignore) uploads only
what the build needs.

```sh
gcloud services enable run.googleapis.com artifactregistry.googleapis.com \
  cloudbuild.googleapis.com secretmanager.googleapis.com
gcloud artifacts repositories create tintype --repository-format=docker --location="$REGION"
gcloud builds submit --config deploy/cloudbuild.yaml --substitutions=_IMAGE="$IMAGE"
```

On the default `e2-standard-2` machine, the release build takes most of the
config's one-hour timeout. Cloud Build's free monthly build-minutes cover it.

Artifact Registry's first 0.5 GB a month is free. Add a cleanup policy so old
images don't build up.

## 2. Service accounts

Each service runs as its own account with no project roles. Only the API's
account can read the token.

```sh
gcloud iam service-accounts create tintype-api
gcloud iam service-accounts create tintype-demo
```

## 3. The token-protected API

Create the token in Secret Manager. Never put it in the repository, an image
layer, a deploy flag or a log.

```sh
openssl rand -hex 32 | tr -d '\n' | gcloud secrets create tintype-api-token --data-file=-
gcloud secrets add-iam-policy-binding tintype-api-token \
  --member="serviceAccount:tintype-api@$PROJECT.iam.gserviceaccount.com" \
  --role=roles/secretmanager.secretAccessor
```

Deploy the service. Cloud Run lets every caller through, and the server
answers 401 unless the request carries the bearer token.

```sh
gcloud run deploy tintype-api --image "$IMAGE" --region "$REGION" \
  --service-account "tintype-api@$PROJECT.iam.gserviceaccount.com" \
  --args=serve,--bind,0.0.0.0:8080,--device,cpu \
  --set-secrets=TINTYPE_API_TOKEN=tintype-api-token:latest \
  --cpu=2 --memory=2Gi --concurrency=4 --max-instances=2 --min-instances=0 \
  --timeout=120 --execution-environment=gen2 --allow-unauthenticated
```

How these flags cap spending:

- Concurrency stays at 4 because the server answers 503 beyond four active
  requests.
- `--max-instances` caps the bill.
- The 120-second timeout covers the server's own deadlines: 20 s to upload,
  then 90 s for queueing and inference.

Limits that differ from running the server yourself:

- **Request size.** Cloud Run limits HTTP/1 request bodies to 32 MiB, so a
  batch can't reach the server's own 64 MiB limit here.
- **Videos.** The API writes uploaded videos to a temporary file while it
  scores them. On Cloud Run that file lives in the instance's memory, and the
  server deletes it afterwards.

Don't link to the API's URL from anywhere public. To read the token:

```sh
gcloud secrets versions access latest --secret=tintype-api-token
```

## 4. The public demo

The demo runs the same binary with `--demo` (see the
[API reference](API.md#public-demo-mode)). It has no token, and it gets no
access to the secret.

Its limits are compiled in, in `src/demo.rs`, and the server won't start
without them. In memory, the server keeps only a keyed hash of each caller's
network, and it logs only aggregate counts. It does not load the video model,
so no upload is ever written to disk.

```sh
gcloud run deploy tintype-demo --image "$IMAGE" --region "$REGION" \
  --service-account "tintype-demo@$PROJECT.iam.gserviceaccount.com" \
  --args=serve,--bind,0.0.0.0:8080,--turbo,--demo,--demo-origin,"$SITE_ORIGIN",--client-address,last-forwarded-for \
  --cpu=2 --memory=1Gi --concurrency=4 --max-instances=1 --min-instances=0 \
  --timeout=60 --execution-environment=gen2 --allow-unauthenticated
```

`--turbo` is used here because the instances measured in us-central1 ran turbo's VNNI kernel; check yours first ([Turbo on Cloud Run](#turbo-on-cloud-run)).

`--max-instances=1` is the hard ceiling on cost. It also keeps the rate limits
in a single process, so they apply across all visitors.

How the server works out the caller's address:

- A run.app URL has no load balancer in front. The TCP peer address is
  Google's front end, not the caller.
- The front end appends the caller's address as the last `X-Forwarded-For`
  entry, and keeps anything the caller sent in front of it. That is why
  `--client-address last-forwarded-for` reads only the last entry.
- Google's documentation doesn't state this for Cloud Run. Check it on every
  new deployment ([Verify](#6-verify)).

What the rate limits don't guarantee:

- **Restarts reset the counts.** The counts live only in memory, so they start
  again when the instance stops (after about 15 idle minutes) or is replaced.
  The instance cap, not the limits, bounds the bill.
- **One instance cap can briefly be exceeded.** Cloud Run may run more than
  `--max-instances` for a short time during traffic spikes.

Once you have the demo's URL, set it as `data-endpoint` on `#demo` in
`site/index.html`.

## 5. Keep request logs out of storage

By default, Cloud Run writes a request log entry for every call to
`run.googleapis.com/requests`, including the caller's IP address and user
agent. Cloud Run itself has no setting to turn this off. The supported way to
stop storing these entries is an exclusion on the project's `_Default` log
sink. Excluded entries are neither stored nor billed. The `_Required` sink
holds only audit logs.

```sh
gcloud logging sinks update _Default --add-exclusion=name=tintype-request-logs,filter='resource.type="cloud_run_revision" AND log_id("run.googleapis.com/requests") AND resource.labels.service_name=("tintype-demo" OR "tintype-api")'
```

The container's own output stays in the logs. For the demo, that is:

- the start-up line;
- an hourly line of counts (photos scored, and refusals by limit).

Data Access audit logs are off by default. Cloud Run doesn't audit-log service
invocations.

## 6. Verify

Check the deployment before you link to it:

```sh
DEMO=$(gcloud run services describe tintype-demo --region "$REGION" --format='value(status.url)')
curl -si -H "Origin: $SITE_ORIGIN" -F file=@site/assets/demo.png "$DEMO/v1/detect" | grep -iE '^(HTTP|ratelimit|access-control|cache-control)'
```

1. **CORS and the rate-limit headers.**
   - The response must show `Access-Control-Allow-Origin: $SITE_ORIGIN`,
     `Cache-Control: no-store` and the `RateLimit` fields.
   - A request from another origin must get 403.
2. **Forged addresses get no new allowance.**
   - Send five more photos, each with a different `X-Forwarded-For` value made
     up by the client.
   - `RateLimit-Remaining` must keep falling, and the sixth photo in the window
     must get 429 with `Retry-After`.
3. **A different network gets its own allowance.** Repeat from a different
   network, such as a phone hotspot. It must start at a full allowance.
4. **The logs are clean.** After a few requests, wait a couple of minutes, then
   read everything the project stored for both services:

   ```sh
   gcloud logging read 'resource.type="cloud_run_revision" AND resource.labels.service_name=("tintype-demo" OR "tintype-api")' --freshness=1h --format=json > logs.json
   jq -r '.[].logName' logs.json | sort | uniq -c
   grep -cE '"remoteIp"|"userAgent"|"requestUrl"' logs.json   # must print 0
   ```

   Also search `logs.json` for your own IP address and user agent. Neither
   should appear.
5. **The API needs its token.**
   - Without the token, `POST /v1/detect` must return 401.
   - With `Authorization: Bearer <token>`, it must score the photo.

The repository's tests cover the same behaviour without a cloud: the limiter,
CORS, the size and type checks, and refusing to start without limits
(`cargo test --release demo`).

To check that the demo never writes an upload to disk, run it on macOS with
every file write denied. It must still score photos:

```sh
printf '(version 1)\n(allow default)\n(deny file-write*)\n(allow file-write* (literal "/dev/null"))\n' > /tmp/deny-writes.sb
sandbox-exec -f /tmp/deny-writes.sb target/release/tintype serve --device cpu --bind 127.0.0.1:8791 --demo --demo-origin http://localhost:8000
curl -s -F file=@site/assets/demo.png http://127.0.0.1:8791/v1/detect
```

As a control, run the API (without `--demo`) under the same profile. A video
upload then fails, because its temporary file can't be written.

## 7. Budget alert

A budget sends alerts; it does not cap spending. The `--max-instances` settings
are the cap.

```sh
gcloud billing budgets create --billing-account="$BILLING_ACCOUNT" \
  --display-name=tintype --budget-amount=25USD \
  --filter-projects="projects/$PROJECT" \
  --threshold-rule=percent=0.5 --threshold-rule=percent=0.9 \
  --threshold-rule=percent=1.0,basis=forecasted-spend
```

The most each service can cost is set by its instance cap:

- For instances that are busy around the clock with 2 vCPU, Tier 1
  request-based prices come to about $125 a month per instance, plus memory
  and $0.40 per million requests.
- The demo's daily cap of 2,000 photos keeps real use within the free
  allowance. A flood of refused requests can still keep its single instance
  busy.

## The daily capacity and the budget

The demo's daily cap across all visitors (`daily_total` in `src/demo.rs`) is
set from the $25 monthly budget. The arithmetic, with Tier 1 request-based
prices:

- **Time per photo.** A 2 vCPU, 1 GiB instance spends about 0.85 s on each
  photo in turbo mode. That figure was measured from a client on 2026-11-16
  and includes the network.
- **Cost per photo.** 0.85 s × (2 × $0.000024 + $0.0000025), plus
  $0.0000004 for the request, is about $0.000043.
- **What $25 buys.** $25 a month at that cost is about 577,000 photos, or
  about 19,000 a day.
- **The cap.** The cap is 15,000 a day. That leaves a margin for cold starts,
  refused requests and slow uploads.
- **The free allowance.** The calculation ignores Cloud Run's free monthly
  allowance. That allowance is shared across the billing account, so it is
  extra margin rather than part of the budget.

If you change the instance size or the engine, measure again and change the
constant. The website does not state the number. When the cap is reached, the
page explains that the demo has a hosting budget and links to the install
instructions.

## Turbo on Cloud Run

When turbo mode loads, the server logs which eight-bit kernel it runs, for
example `turbo mode: photos run on the eight-bit engine, kernel: AVX-512 VNNI
...`. To check an instance, send one photo to the API with `?mode=turbo`, then
read the service's logs.

| Kernel in the log | Meaning | What to do |
|---|---|---|
| AVX-VNNI or AVX-512 VNNI | The instance's CPU has VNNI | Turbo is faster. On 2026-11-16 in us-central1: about 0.82 s against 1.10 s per photo, file to verdict, from a client |
| AVX2 | No VNNI; the server also prints a note | Turbo is slower than standard. Leave it off |

Cloud Run doesn't promise one CPU model, so check again after redeploying.

## Remove it

```sh
gcloud run services delete tintype-demo --region "$REGION"
gcloud run services delete tintype-api --region "$REGION"
gcloud secrets delete tintype-api-token
gcloud artifacts repositories delete tintype --location="$REGION"
gcloud logging sinks update _Default --remove-exclusions=tintype-request-logs
```
