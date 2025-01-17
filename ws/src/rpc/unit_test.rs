/// tests
#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use chrono::Utc;
    use serde_json;

    use rust_ocpp::v2_0_1::datatypes::charging_station_type::ChargingStationType;
    use rust_ocpp::v2_0_1::enumerations::boot_reason_enum_type::BootReasonEnumType;
    use rust_ocpp::v2_0_1::enumerations::connector_status_enum_type::ConnectorStatusEnumType;
    use rust_ocpp::v2_0_1::enumerations::registration_status_enum_type::RegistrationStatusEnumType;
    use rust_ocpp::v2_0_1::messages::boot_notification::BootNotificationRequest;
    use rust_ocpp::v2_0_1::messages::boot_notification::BootNotificationResponse;
    use rust_ocpp::v2_0_1::messages::heartbeat::HeartbeatRequest;
    use rust_ocpp::v2_0_1::messages::heartbeat::HeartbeatResponse;
    use rust_ocpp::v2_0_1::messages::status_notification::StatusNotificationRequest;

    use crate::rpc::enums::BootNotificationKind;
    use crate::rpc::enums::OcppActionEnum;
    use crate::rpc::enums::OcppPayload;
    use crate::rpc::errors::RpcErrorCodes;
    use crate::rpc::messages::{OcppCall, OcppMessageType};
    use serde::Deserialize;

    #[test]
    fn serialize_deserialize_json_comparison() {
        let json = r#"[2,"19223201","BootNotification",{"chargingStation":{"model":"SingleSocketCharger","vendorName":"VendorX"},"reason":"PowerUp"}]"#;

        let ocpp_message = serde_json::from_str::<OcppMessageType>(&json).unwrap();

        let back2json = serde_json::to_string(&ocpp_message).unwrap();

        assert_eq!(json, back2json);
    }

    #[test]
    fn deserialize_call() {
        let bnr = BootNotificationRequest {
            reason: BootReasonEnumType::PowerUp,
            charging_station: ChargingStationType {
                model: "SingleSocketCharger".to_string(),
                vendor_name: "VendorX".to_string(),
                serial_number: None,
                firmware_version: None,
                modem: None,
            },
        };

        assert_eq!(bnr.reason, BootReasonEnumType::PowerUp);

        let json = serde_json::to_string(&bnr).unwrap();

        println!("Serialized to {}", json);
    }

    // Testing all error codes are correct for RpcErrorCodes For OCPP 2.0.1
    // "4.3. RPC Framework Error Codes" in "OCPP-2.0.1_part4_ocpp-j-specification.pdf"
    #[test]
    fn test_rpc_error_codes_ocpp_2_0_1() {
        // format validation error
        let format_violation_err = RpcErrorCodes::FormatViolation;
        assert_eq!(
            format_violation_err.description(),
            "Payload for Action is syntactically incorrect"
        );

        // generic error
        let generic_err = RpcErrorCodes::GenericError;
        assert_eq!(
            generic_err.description(),
            "Any other error not covered by the more specific error codes in this table"
        );

        // internal error
        let internal_err = RpcErrorCodes::InternalError;
        assert_eq!(internal_err.description(), "An internal error occurred and the receiver was not able to process the requested Action successfully");

        // message type not supported
        let messagetypenotsupported_err = RpcErrorCodes::MessageTypeNotSupported;
        assert_eq!(messagetypenotsupported_err.description(), "A message with an Message Type Number received that is not supported by this implementation.");

        // not implemented
        let not_implemented_err = RpcErrorCodes::NotImplemented;
        assert_eq!(
            not_implemented_err.description(),
            "Requested Action is not known by receiver"
        );

        // not supported
        let not_supported = RpcErrorCodes::NotSupported;
        assert_eq!(
            not_supported.description(),
            "Requested Action is recognized but not supported by the receiver"
        );

        // cccurrence constraint violation
        let occurrence_constraint_violation_err = RpcErrorCodes::OccurrenceConstraintViolation;
        assert_eq!(occurrence_constraint_violation_err.description(), "Payload for Action is syntactically correct but at least one of the fields violates occurrence constraints");

        // property constraint violation
        let property_constraint_violation_err = RpcErrorCodes::PropertyConstraintViolation;
        assert_eq!(
            property_constraint_violation_err.description(),
            "Payload is syntactically correct but at least one field contains an invalid value"
        );

        // protocol error
        let protocol_error_err = RpcErrorCodes::ProtocolError;
        assert_eq!(
            protocol_error_err.description(),
            "Payload for Action is not conform the PDU structure"
        );

        // rpc framework error
        let rpc_framework_error = RpcErrorCodes::RpcFrameworkError;
        assert_eq!(rpc_framework_error.description(), "Content of the call is not a valid RPC Request, for example: MessageId could not be read.");

        // security error
        let security_error_err = RpcErrorCodes::SecurityError;
        assert_eq!(security_error_err.description(), "During the processing of Action a security issue occurred preventing receiver from completing the Action successfully");

        // type constraint violation
        let type_constraint_violation = RpcErrorCodes::TypeConstraintViolation;
        assert_eq!(type_constraint_violation.description(), "Payload for Action is syntactically correct but at least one of the fields violates data type constraints (e.g. \"somestring\": 12)");
    }

    #[test]
    fn b01_cold_boot_charging_station() {
        // B01 - Cold Boot Charging Station

        // The Charging Station sends BootNotificationRequest to the CSMS.
        let call_with_bootnotification_request = r#"
                [
                    2,
                    "19223201",
                    "BootNotification",
                    {
                        "reason": "PowerUp",
                        "chargingStation": {
                            "model": "SingleSocketCharger",
                            "vendorName": "VendorX"
                        }
                    }
                ]
            "#;

        let call: Result<OcppMessageType, _> =
            serde_json::from_str(call_with_bootnotification_request);

        assert_eq!(call.is_ok(), true);

        // The CSMS returns with BootNotificationResponse with the status Accepted.
        let bn_res = BootNotificationResponse {
            current_time: Utc::now(),
            interval: 1,
            status: RegistrationStatusEnumType::Accepted,
            status_info: None,
        };

        assert_eq!(bn_res.status, RegistrationStatusEnumType::Accepted);

        // The Charging Station sends StatusNotificationRequest to the CSMS for
        // each Connector. If the status was set to Unavailable or Reserved from
        // the CSMS prior to the (re)boot, the Connector should return to this
        // status, otherwise the status should be Available or, when it resumes
        // a transaction that was ongoing, the status should be Occupied.

        let status_notification_req = StatusNotificationRequest {
            timestamp: Utc::now(),
            connector_status: ConnectorStatusEnumType::Available,
            evse_id: 6464,
            connector_id: 6464,
        };

        assert_eq!(
            status_notification_req.connector_status,
            ConnectorStatusEnumType::Available
        );

        // The Charging Station sends HeartbeatRequest to the CSMS.

        let heart_beat_req: HeartbeatRequest = HeartbeatRequest {};
        let heart_beat_res: HeartbeatResponse = HeartbeatResponse {
            current_time: Utc::now(),
        };

        println!("heart beat request: {:?}", heart_beat_req);
        println!("heart beat response: {:?}", heart_beat_res);
    }

    #[test]
    fn test_call() {
        let bootnotificationrequest_json = r#"
                [
                    2,
                    "19223201",
                    "BootNotification",
                    {
                        "reason": "PowerUp",
                        "chargingStation": {
                            "model": "SingleSocketCharger",
                            "vendorName": "VendorX"
                        }
                    }
                ]
            "#;
        // get ocpp message
        let ocpp_msg: Result<OcppMessageType, _> =
            serde_json::from_str(&bootnotificationrequest_json);

        println!("{ocpp_msg:?}");
        assert_eq!(ocpp_msg.is_ok(), true);
    }

    #[test]
    fn test_call_result() {
        let callresult_json = r#"
            [
                3,
                "19223201",
                {
                    "currentTime": "2013-02-01T20:53:32.486Z",
                    "interval": 300,
                    "status": "Accepted"
                }
            ]
            "#;
        // get ocpp message
        let ocpp_msg: Result<OcppMessageType, _> = serde_json::from_str(&callresult_json);

        println!("{ocpp_msg:?}");
        assert_eq!(ocpp_msg.is_ok(), true);
    }

    #[test]
    fn test_call_error() {
        let callerror_json = r#"
            [
                4,
                "162376037",
                "NotSupported",
                "SetDisplayMessageRequest not implemented",
                {}
            ]
            "#;
        // get ocpp message
        let ocpp_msg: Result<OcppMessageType, _> = serde_json::from_str(&callerror_json);

        assert_eq!(ocpp_msg.is_ok(), true);
    }

    #[test]
    fn test_type_cast_to_call() {
        let bootnotificationrequest_json = r#"
                [
                    2,
                    "19223201",
                    "BootNotification",
                    {
                        "reason": "PowerUp",
                        "chargingStation": {
                            "model": "SingleSocketCharger",
                            "vendorName": "VendorX"
                        }
                    }
                ]
            "#;
        // get ocpp message
        let ocpp_msg: Result<OcppMessageType, _> =
            serde_json::from_str(&bootnotificationrequest_json);

        assert_eq!(ocpp_msg.is_ok(), true);
    }
}
