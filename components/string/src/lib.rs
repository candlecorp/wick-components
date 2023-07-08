mod wick {
    wick_component::wick_import!();
}
use wick::*;
use wick_component::packet::Packet;

#[async_trait::async_trait(?Send)]
impl ConcatenateOperation for Component {
    type Error = anyhow::Error;
    type Outputs = concatenate::Outputs;
    type Config = concatenate::Config;

    async fn concatenate(
        left: WickStream<Packet>,
        right: WickStream<Packet>,
        outputs: Self::Outputs,
        _ctx: Context<Self::Config>,
    ) -> Result<(), Self::Error> {
        let (_, _, mut outputs) = handle_stream(None, None, left, right, outputs).await;
        outputs.output.done();

        Ok(())
    }
}

#[cfg_attr(not(target_family = "wasm"), async_recursion::async_recursion)]
#[cfg_attr(target_family = "wasm", async_recursion::async_recursion(?Send))]
async fn handle_stream(
    last_left: Option<String>,
    last_right: Option<String>,
    mut l_stream: WickStream<Packet>,
    mut r_stream: WickStream<Packet>,
    mut outputs: concatenate::Outputs,
) -> (WickStream<Packet>, WickStream<Packet>, concatenate::Outputs) {
    loop {
        match (&last_left, &last_right) {
            (Some(_), Some(_)) => {
                unreachable!()
            }
            (Some(left), None) => {
                let right = r_stream.next().await;
                if right.is_none() {
                    break;
                }
                let right = right.unwrap();
                let right = propagate_if_error!(right, outputs, continue);
                if right.is_open_bracket() {
                    outputs.broadcast_open();
                    (l_stream, r_stream, outputs) =
                        handle_stream(Some(left.clone()), None, l_stream, r_stream, outputs).await;
                    outputs.broadcast_close();
                } else if right.is_close_bracket() || right.is_done() {
                    break;
                } else {
                    let right: String = propagate_if_error!(right.decode(), outputs, continue);
                    outputs.output.send(&format!("{}{}", left, right));
                }
            }
            (None, Some(right)) => {
                let left = l_stream.next().await;
                if left.is_none() {
                    break;
                }
                let left = left.unwrap();
                let left = propagate_if_error!(left, outputs, continue);
                if left.is_open_bracket() {
                    outputs.broadcast_open();
                    (l_stream, r_stream, outputs) =
                        handle_stream(None, Some(right.clone()), l_stream, r_stream, outputs).await;
                    outputs.broadcast_close();
                } else if left.is_close_bracket() || left.is_done() {
                    break;
                } else {
                    let left: String = propagate_if_error!(left.decode(), outputs, continue);
                    outputs.output.send(&format!("{}{}", left, right));
                }
            }
            (None, None) => {
                let right = r_stream.next().await;
                let left = l_stream.next().await;
                if right.is_none() || left.is_none() {
                    // nothing more we can do
                    break;
                }
                let left = left.unwrap();
                let right = right.unwrap();
                let left = propagate_if_error!(left, outputs, continue);
                let right = propagate_if_error!(right, outputs, continue);
                match (left.is_open_bracket(), right.is_open_bracket()) {
                    (true, true) => {
                        outputs.broadcast_open();
                        (l_stream, r_stream, outputs) =
                            handle_stream(None, None, l_stream, r_stream, outputs).await;
                        outputs.broadcast_close();
                    }
                    (true, false) => {
                        if right.is_close_bracket() || right.is_done() {
                            break;
                        }

                        let right: String = propagate_if_error!(right.decode(), outputs, continue);
                        outputs.broadcast_open();
                        (l_stream, r_stream, outputs) =
                            handle_stream(None, Some(right), l_stream, r_stream, outputs).await;
                        outputs.broadcast_close();
                    }
                    (false, true) => {
                        if left.is_close_bracket() || left.is_done() {
                            break;
                        }
                        let left: String = propagate_if_error!(left.decode(), outputs, continue);
                        outputs.broadcast_open();
                        (l_stream, r_stream, outputs) =
                            handle_stream(Some(left), None, l_stream, r_stream, outputs).await;
                        outputs.broadcast_close();
                    }
                    (false, false) => {
                        if left.is_close_bracket() || left.is_done() {
                            break;
                        }
                        if right.is_close_bracket() || right.is_done() {
                            break;
                        }
                        let left: String = propagate_if_error!(left.decode(), outputs, continue);
                        let right: String = propagate_if_error!(right.decode(), outputs, continue);
                        outputs.output.send(&format!("{}{}", left, right));
                    }
                }
            }
        }
    }

    (l_stream, r_stream, outputs)
}
