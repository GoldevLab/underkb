#!/bin/sh
# The volume hides the image's /data and is root-owned. Fix that, then drop
# privileges so the local counter file survives a deploy when Turso is unset.
set -eu
mkdir -p /data
chown 65532:65532 /data
exec setpriv --reuid=65532 --regid=65532 --clear-groups /app/server
