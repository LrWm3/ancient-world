// Define shared scalar parameters in their owning subsystem. The literal spelling
// is emitted unchanged into WGSL; no runtime float formatting or new buffer ABI.
// Use WGSL-compatible literals without Rust-only suffixes or separators.
// CPU f64 parameters retain double precision; WGSL uses the same spelling as f32.
macro_rules! shared_shader_parameters {
    (@wgsl_type f64) => { "f32" };
    (@wgsl_type $ty:ident) => { stringify!($ty) };
    ($source:ident { $($vis:vis const $name:ident: $ty:ident = $value:literal;)* }) => {
        $($vis const $name: $ty = $value;)*
        pub(crate) const $source: &str = concat!($(
            "const ", stringify!($name), ": ", crate::shared_shader_parameters!(@wgsl_type $ty), " = ",
            stringify!($value), ";\n",
        )*);
    };
}
pub(crate) use shared_shader_parameters;

pub mod catalog;
pub mod config;
pub mod gpu;
pub mod grid;
pub mod storage;
pub mod viewer;

pub mod ecology;

pub mod region;

pub mod civilization;

pub mod economy;

pub mod road_upkeep;
pub mod society;

pub mod leadership;
pub mod politics;

pub mod governance;
pub mod offices;

pub mod shipping;

pub mod expeditions;

pub mod discoveries;

pub mod hazards;

pub mod culture;

pub mod agriculture;

pub mod production;

pub mod export_contracts;

pub mod relocation;

pub mod relief;
pub mod religious_relief;

pub mod social_state;

pub mod household_economy;

pub mod institution_capacity;
pub mod institution_funding;
pub mod institution_services;
pub mod institution_succession;

pub mod regional_mining;
pub mod resources;

pub mod environmental_returns;

pub mod metallurgy;

pub mod tool_access;

pub mod social_memory;

pub mod expedition_heritage;

pub mod occupation;

pub mod local_places;

pub mod enterprises;

pub mod history_timeline;

pub mod faction_interests;

pub mod facilities;
pub mod materials;

pub mod naming;

pub mod systems;

pub mod spatial;

pub mod territory;

mod history_atlas;

pub mod vessels;

pub mod civic_petitions;

pub mod history_environment;

pub mod navigation;

mod freight;

mod labor;
mod learning_resolution;
pub mod service_allocation;

pub mod participation;

pub mod military;

pub mod domestic;

pub mod population_registry;

mod individual_demography;

pub mod resolution;

mod workshop_resolution;

pub mod heritage_renown;

pub mod trade_contact;

pub mod agriculture_participation;

mod kin_support;

mod military_supply;

#[cfg(test)]
mod continuity_fixture;
pub mod institution_relocation;

pub mod artifact_petitions;

pub mod route_warnings;

pub mod contagion;

pub mod peace;

pub mod siege;
