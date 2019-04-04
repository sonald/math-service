#!/usr/bin/env bash

if [[ x$1 == xlocal ]]; then
    cargo run --features  'local' --bin paint-service
else
    cargo build --release --features 'service'
    rsync -avzP target/release/paint-service  root@scloud:
    rsync -avzP index.html root@scloud:/web/api.sonald.me/

fi
