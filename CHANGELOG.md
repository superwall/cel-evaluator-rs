# CHANGELOG

## 0.1.10

## Enhancements

- Updates github workflow for the renaming of the iOS repository.

## 0.1.9

## Enhancements

- Added returning of a JSON encoded `Result<PassableValue,String>` from the exposed methods instead of relying on panics.
  Example JSON:
  - Error: `{"Err":"No such key: should_display"}`
  - Ok: `{"Ok":{"type":"bool","value":true}}`

## Fixes

- Fixed a bug where getting properties from `device` would panic when `device` functions were defined