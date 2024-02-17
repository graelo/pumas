//! Define convenient units using the `si_scale` crate.

use si_scale::scale_fn;

// defines the `watts()` function: 18 W
scale_fn!(watts,
    base: B1000,
    constraint: UnitAndBelow,
    mantissa_fmt: "{:.0}",
    unit: "W",
    doc: "Return a string with the value and its si-scaled unit of watts.");

// defines the `watts2()` function: 18.65 W
scale_fn!(watts2,
    base: B1000,
    constraint: UnitAndBelow,
    mantissa_fmt: "{:.2}",
    unit: "W",
    doc: "Return a string with the value and its si-scaled unit of watts.");

// defines the `percent1()` function: 23.6 %
scale_fn!(percent1,
    base: B1000,
    constraint: UnitOnly,
    mantissa_fmt: "{:.1}",
    unit: "%",
    doc: "Return a string with the value and its si-scaled percentage.");

// defines the `mhz()` function: 972 MHz
scale_fn!(mhz,
    base: B1000,
    constraint: UnitOnly,
    mantissa_fmt: "{:.0}",
    unit: "MHz",
    doc: "Return a string with the value and its si-scaled unit of MHz.");

// defines the `bibytes1()` function: 9.56 GB
scale_fn!(bibytes1,
    base: B1024,
    constraint: UnitAndAbove,
    mantissa_fmt: "{:.1}",
    unit: "B",
    doc: "Return a string with the value and its si-scaled unit of bibytes.");
