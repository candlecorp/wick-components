
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

wickdoc:
  #!/usr/bin/env bash
  shopt -s nocaseglob
  for i in components/*; do
    pushd $i > /dev/null
    if test -f "component.wick"; then
      wick invoke -q ../wickdoc/component.wick generate_readme --values -- --input=@component.wick > README.md
    else
      echo "Warning: component $i does not have a component.wick file. Skipping wickdoc..."
    fi
    popd > /dev/null
  done

readme:
  echo "# Wick components" > README.md
  echo "" >> README.md
  echo "This repository contains the following components:" >> README.md
  echo "" >> README.md
  for i in components/*; do echo "- [$(wick query -f $i/component.wick --type yaml name)](./$i)" >> README.md; done

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
