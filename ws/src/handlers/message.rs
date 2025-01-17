use futures::stream::SplitSink;
use log::warn;
use log::info;

use warp::ws::{Message, WebSocket};

use crate::{handlers::error::handle_error, rpc::messages::OcppMessageType};
use crate::rpc::enums::OcppActionEnum;
use crate::rpc::messages::{OcppCall, OcppCallError, OcppCallResult};

use super::response::handle_response;


/*
    The job of handle_message is to:
    Validate that:
        1. incoming transmission is of type text
        2. Cast to correct Call type
*/
pub async fn handle_message(msg: Message, tx: &mut SplitSink<WebSocket, Message>) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        warn!("Client sent non-text message");
        handle_error(
            Message::text("Failed to parse incoming message".to_string()),
            tx,
        )
        .await;
        return;
    };

    // serialize or die
    let ocpp_message: OcppMessageType = match serde_json::from_str(msg) {
        Ok(o) => o,
        Err(_) => {
            handle_error(Message::text("failed to parse payload"), tx).await;
            return;
        }
    };

    match ocpp_message {
            OcppMessageType::Call(_, _, _, _) => {
                let call: Result<OcppCall, _> = ocpp_message.clone().try_into();
                match call {
                    Ok(ok_call) => {
                        // Do some more testing
                        // assert_eq!(ok_call.action, OcppActionEnum::BootNotification);
                        // assert_eq!(ok_call.message_type_id, 2);
                        // assert_eq!(serde_json::to_string(&ok_call).unwrap(), req);
                        match ok_call.action {
                            OcppActionEnum::BootNotification => {
                                // is new device ok?
                                info!("New charging station booted");
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        panic!("Failed to parse Call")
                    }
                };
            }
            OcppMessageType::CallResult(_, _, _) => {
                let call_result: Result<OcppCallResult, _> = ocpp_message.clone().try_into();
                match call_result {
                    Ok(ok_callresult) => {
                        info!("Got a CallResult: {ok_callresult:#?}");
                    }
                    _ => {
                        panic!("Failed to parse CallResult")
                    }
                };
            }
            OcppMessageType::CallError(_, _, _, _, _) => {
                let call_error: Result<OcppCallError, _> = ocpp_message.clone().try_into();
                match call_error {
                    Ok(ok_callerror) => {
                        info!("Got a CallError: {ok_callerror:#?}");
                    }
                    _ => panic!("Failed to parse CallError"),
                }
            }
        }



    handle_response(
        Message::text(serde_json::to_string(&ocpp_message).unwrap()),
        tx,
    )
    .await;

    // what type of payload did we get?

    // match payload {
    //     // request?
    //     Payload::Request(message_type_id, message_id, action, payload) => {
    //         let call = Call::new(message_type_id, message_id, action, payload);
    //         handle_call(call, tx).await;
    //     }
    //     // response?
    //     Payload::Response(message_type_id, message_id, payload) => {
    //         let call_result = CallResult::new(message_type_id, message_id, payload);
    //         handle_callresult(call_result, tx).await;
    //     }
    //     // error?
    //     Payload::Error(
    //         message_type_id,
    //         message_id,
    //         error_code,
    //         error_description,
    //         error_details,
    //     ) => {
    //         let call_error = CallError::new(
    //             message_type_id,
    //             message_id,
    //             error_code,
    //             error_description,
    //             error_details,
    //         );
    //         handle_callerror(call_error, tx).await;
    //     }
    // }

    // We got some text, let's try to deserialize to json
    // let json = if let Ok(v) = serde_json::Value::from_str(msg) {
    //     v
    // } else {
    //     warn!("Client did not send valid json: {}", msg);
    //     handle_error(Message::text(format!("Expected json, got {}", msg)), tx).await;
    //     return;
    // };

    // validate that json is of type array
    // if !json.is_array() {
    //     handle_error(Message::text(format!("Expected array, got: {}", msg)), tx).await;
    //     return;
    // }

    /*
        In OCPP_2_01 the message_type_id is encoded in the 0th field in the
        json array. a message_type_id is either:
        - 2: A Call
        - 3: A CallResult
        - 4: A CallError

        The OCPP_2_0_1 description:
            To identify the type of message one of the following Message Type
            Numbers MUST be used.
            | MessageType | MessageTypeNumber | Description |
            | --- | --- | --- |
            | CALL | 2 | Request message |
            | CALLRESULT | 3 | Response message |
            | CALLERROR | 4 | Error response to a request message |
            When a server receives a message with a Message Type Number not in
            this list, it SHALL ignore the message payload. Each message type
            may have additional required fields.
    */
    // let message_type_id = match validate_message_type_id(&json).await {
    //     Ok(o) => o,
    //     Err(e) => {
    //         handle_error(e, tx).await;
    //         return;
    //     }
    // };

    /*
        The message ID

        The message ID serves to identify a request. A message ID for any CALL
        message MUST be different from all message IDs previously used by the
        same sender for any other CALL messages on the same WebSocket
        connection. A message ID for a CALLRESULT or CALLERROR message MUST be
        equal to that of the CALL  message that the CALLRESULT or CALLERROR
        message is a response to.

        TODO: How do we verify that the message id has not been previously used?
    */
    // let message_id = match validate_message_id(&json).await {
    //     Ok(o) => o,
    //     Err(e) => {
    //         handle_error(e, tx).await;
    //         return;
    //     }
    // };
    // info!("Got message_id: {}", message_id);

    // try to deserialize json to a Call, CallResult or CallError
    // if message_type_id == 2 {
    //     // It's a Call
    //     let call: Call = match serde_json::from_str(&msg.to_string()) {
    //         Ok(o) => o,
    //         Err(e) => {
    //             handle_error(Message::text(e.to_string()), tx).await;
    //             return;
    //         }
    //     };
    //     handle_call(call, tx).await;
    // } else if message_type_id == 3 {
    //     // It's a CallResult
    //     let call_result: CallResult = match serde_json::from_str(&msg.to_string()) {
    //         Ok(o) => o,
    //         Err(_) => {
    //             handle_error(Message::text("Could not deserialize CallResult"), tx).await;
    //             return;
    //         }
    //     };
    //     info!("got {:?}", call_result);
    //     handle_response(Message::text("Responding to CallResult"), tx).await;
    // } else if message_type_id == 4 {
    //     // It's a CallError
    //     let call_error: CallError = match serde_json::from_str(&msg.to_string()) {
    //         Ok(o) => o,
    //         Err(_) => {
    //             handle_error(Message::text("Could not deserialize CallResultError"), tx).await;
    //             return;
    //         }
    //     };
    //     info!("got {:?}", call_error);
    //     handle_response(Message::text("Responding to CallError"), tx).await;
    // };
}
