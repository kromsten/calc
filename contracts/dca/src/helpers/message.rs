use cosmwasm_std::{Event, StdError, StdResult};

pub fn get_attribute_in_event(
    events: &[Event],
    event_type: &str,
    attribute_key: &str,
) -> StdResult<String> {
    let events_with_type = events.iter().filter(|event| event.ty == event_type);

    let attribute = events_with_type
        .into_iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attribute| attribute.key == attribute_key)
        .ok_or(StdError::generic_err(format!(
            "unable to find {} attribute in {} event",
            attribute_key, event_type
        )))?;

    Ok(attribute.value.clone())
}
