# SunKey

## Building the APK

```
docker run \
  --rm \
  -v $(pwd):/root/src \
  -v /tmp/registry\":/usr/local/cargo/registry\" \
  -w /root/src -it \
  notfl3/cargo-apk /bin/bash
```

and in the shell:

```
cargo quad-apk build --release
```
