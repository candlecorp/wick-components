use crate::actions::unzip_service::unzip::*;

pub(crate) async fn task(input: Inputs) -> Result<Outputs, crate::Error> {
    let mut zip = stream_unzip::ZipReader::new();
    let mut stream = reader::read(reader::read::Inputs {
        source: input.source,
    });

    while let Some(Ok(payload)) = stream.next().await {
        zip.update(payload);

        let entries = zip.drain_entries();
        for entry in entries {
            let expand = entry.inflate().unwrap();
            let (file_tx, file_rx) = Flux::new_channels();

            let _res = writer::write(writer::write::Inputs {
                contents: file_rx,
                dest: expand.name().to_owned(),
            });
            let bytes = expand.data();
            for chunk in bytes.chunks(1024) {
                let _ = file_tx.send(chunk.to_owned().into());
            }
            file_tx.complete();
        }
    }

    Ok(())
}
