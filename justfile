
build:
  for i in components/*; do just $i/build; done

test:
  for i in components/*; do just $i/test; done