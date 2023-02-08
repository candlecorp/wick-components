
build:
  for i in iotas/*; do just $i/build; done

test:
  for i in iotas/*; do just $i/test; done