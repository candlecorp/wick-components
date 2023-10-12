
set positional-arguments

run-child-task task:
  #!/usr/bin/env bash
  shopt -s nocaseglob
  for i in components/*; do
    if test -f "$i/Justfile"; then
      echo "Running $i/{{task}}..."
      # OUTPUT=$(just $i/{{task}} 2>&1)
      just $i/{{task}}
      if [ $? -eq 0 ]; then
        echo "... done"
      else
        # echo "Error: $OUTPUT"
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

publish:
  just run-child-task publish

clean:
  just run-child-task clean
  rm components/*/Cargo.lock


# Quickly generate a new WebAssembly component from a wick template
new-component name lang="rust" path="components":
  echo "Generating new {{lang}} example: {{name}}"
  echo "Note: This is optimized for macOS, if it breaks on your platform, please open an issue"
  mkdir -p {{path}}
  cd {{path}} && cargo generate candlecorp/wick templates/{{lang}} --name {{name}}

  echo "New {{lang}} component generated at {{path}}/{{name}}"
