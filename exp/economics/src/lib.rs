//! Stand-alone process-based economics experiment with CubeCL CPU settlement.
pub mod commitments;
pub mod compute;
pub mod equipment;
pub mod maintenance;
pub mod model;
pub mod planning;
pub mod pools;
pub mod scenario;
pub mod settlement;
pub mod simulation;
pub mod substitution;

#[cfg(test)]
mod tests {
    use cubecl::{cpu::CpuRuntime, prelude::*};

    #[cube(launch)]
    fn add_one(input: &Array<f32>, output: &mut Array<f32>) {
        if ABSOLUTE_POS < input.len() {
            output[ABSOLUTE_POS] = input[ABSOLUTE_POS] + 1.0;
        }
    }

    fn check_kernel<R: Runtime>(device: &R::Device) {
        let client = R::client(device);
        let input = [-2.0_f32, -0.5, 0.0, 1.0, 7.25];
        let input_handle = client.create_from_slice(f32::as_bytes(&input));
        let output_handle = client.empty(core::mem::size_of_val(&input));

        // Deliberately launch more units than elements to exercise the tail guard.
        // SAFETY: Both buffers contain exactly input.len() f32 elements. The
        // kernel guards excess units, and input/output use distinct allocations.
        unsafe {
            add_one::launch::<R>(
                &client,
                CubeCount::Static(2, 1, 1),
                CubeDim::new_1d(4),
                ArrayArg::from_raw_parts(input_handle, input.len()),
                ArrayArg::from_raw_parts(output_handle.clone(), input.len()),
            )
        }

        let bytes = client.read_one(output_handle).expect("CPU kernel readback");
        let actual = f32::from_bytes(&bytes);
        let expected = input.map(|value| value + 1.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn cubecl_cpu_compiles_executes_and_reads_back_kernel() {
        check_kernel::<CpuRuntime>(&Default::default());
    }
}

pub mod currency;
pub mod storage;

pub mod activities;
pub mod crafts;

pub mod exchange;

pub mod trading_scenario;

pub mod forward;

pub mod plots;

pub mod households;

pub mod opportunities;

pub mod membership;

pub mod search;

pub mod finance;

pub mod agreements;

pub mod offers;

pub mod allocation;
pub mod competition;

pub mod pool_market;

pub mod resolution;

pub mod intermediary;

pub mod access_expectations;

pub mod consequence_priority;

pub mod negotiation;

pub mod marketplace;

pub mod zip;

pub mod credit;

pub mod work_choice;

pub mod resale;

pub mod forecast;

pub mod borrowing;

pub mod stock_sale;

pub mod sale_plan;

pub mod joint_plan;

pub mod acquisition;

pub mod need_orders;

pub mod town_market;

pub mod production_market;

pub mod telemetry;

pub mod calibration;

pub mod credit_stress;

pub mod cooperation;

pub mod laws;

pub mod minting;

pub mod recovery;

mod asset_exchange;

pub mod recovery_claims;

pub mod delivery_relief;

pub mod household_governance;

pub mod accounting;
pub mod financial_reporting;

pub mod inventory_accounting;

pub mod process_accounting;
