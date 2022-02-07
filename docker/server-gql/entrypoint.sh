#!/usr/bin/env bash

set -Eeo pipefail

_main() {
	if [ "$1" = 'run' ]; then
    /srv/docstore/bin/gql --config-dir /srv/docstore/etc/gql --run-mode docker run
  fi
}

_main "$@"

