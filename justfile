
set positional-arguments

run-child-task task:
  #!/usr/bin/env bash
  shopt -s nocaseglob
  for i in components/*; do
    if test -f "$i/Justfile"; then
      echo "Running $i/{{task}}..."
      OUTPUT=$(just $i/{{task}} 2>&1)
      if [ $? -eq 0 ]; then
        echo "... done"
      else
        echo "Error: $OUTPUT"
        echo "Failed to build $i."
        exit 1
      fi
    else
      echo "Warning: component $i does not have a Justfile. Skipping..."
    fi
  done

build:
  just run-child-task build

test:
  just run-child-task test

clean:
  just run-child-task clean
  rm components/*/Cargo.lock
