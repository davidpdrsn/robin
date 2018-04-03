# `#[derive(EachVariant)]`

Derive method that returns each variant of an enum

## Sample usage

```rust
#[macro_use]
extern crate enum_each_variant_derive;

#[derive(EachVariant, Eq, PartialEq, Debug)]
enum Thing {
    One,
    Two,
    Three,
    Four,
}

let all: Vec<Thing> = Thing::all_variants();

assert_eq!(all, vec![Thing::One, Thing::Two, Thing::Three, Thing::Four]);
```

## Gotcha

Only works on enums where no variants have associated values. So we wouldn't be able to use it
for this enum:

```rust
enum TrainStatus {
    OnTime,
    DelayedBy(std::time::Duration),
}
```
