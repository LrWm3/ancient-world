import unittest
from evidence import test_counts, summary, stage_passed

class EvidenceTests(unittest.TestCase):
    def test_summaries_include_failures_and_skips(self):
        self.assertEqual(test_counts('test result: FAILED. 2 passed; 1 failed; 8 ignored; 0 measured; 3 filtered out; finished in 1s'),
                         [dict(passed=2, failed=1, ignored=8, measured=0, filtered=3)])
        self.assertEqual(test_counts('compiler error'), [])

    def test_command_failure_and_zero_executed_tests_fail_closed(self):
        self.assertFalse(stage_passed(1, [], False))
        self.assertFalse(stage_passed(0, [], True))
        self.assertFalse(stage_passed(0, [dict(passed=0, failed=0, ignored=12)], True))
        self.assertFalse(stage_passed(0, [dict(passed=2, failed=1)], True))
        self.assertTrue(stage_passed(0, [dict(passed=2, failed=0)], True))

    def test_incomplete_report_cannot_look_passed(self):
        report = summary(dict(profile='focused', status='failed', commit={'output':'abc'}, stages=[{'name':'gpu','status':'failed'}]))
        self.assertIn('status: failed', report)
        self.assertIn('not empirical calibration', report)

class CouplingSummaryTests(unittest.TestCase):
    def test_missing_probes_are_not_zero(self):
        from summarize_coupling import tool_multiplier
        self.assertIsNone(tool_multiplier({'sites':[{}]}))
        self.assertAlmostEqual(tool_multiplier({'sites':[
            {'production_probe':[0.75, 0, 0, 3]},
            {'production_probe':[1, 0, 0, 1]}]}), 0.8125)

    def test_reserve_groups_remain_distinct(self):
        import json
        import tempfile
        from pathlib import Path
        from summarize_coupling import main
        site = dict(population=10, labor_workers=[1]*4, production_kg=1,
                    ration_eaten_kg=[1]*4, tool_stock_kg=1)
        month = dict(sites=[site], economy_residuals=[0], source_residual=0,
                     ecology_relative_error=[0])
        runs = [dict(seed=17, fixed_labor=fixed, tool_fraction=fraction,
                     branches=[dict(months=[month]),dict(months=[month])])
                for fraction in (1,0) for fixed in (False,True)]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'results.json'
            path.write_text(json.dumps(dict(complete=True,gpu='fixture',closure_months=1,runs=runs)))
            main(path)
            report = path.with_suffix('.md').read_text()
            self.assertIn('| 17 | 1 | 0.00 | 0.00 | 0.00 |',report)
            self.assertIn('| 17 | 0 | 0.00 | 0.00 | 0.00 |',report)
            self.assertIn('not recorded',report)
            self.assertNotIn('absolute source residual',report)

class PolicyReferenceTests(unittest.TestCase):
    def test_fixed_control_must_reproduce_prior_measurements(self):
        import copy
        from summarize_coupling import compare_policy
        sample = dict(sites=[dict(population=10)])
        run = dict(seed=17, tool_fraction=1, fixed_labor=True, config={},
                   branches=[dict(closed=False, months=[sample])])
        reference = dict(complete=True, closure_months=1, recovery_months=0, runs=[run])
        current = copy.deepcopy(reference)
        current['runs'][0]['branches'][0]['months'][0]['sites'][0]['food_labor'] = [0]*4
        self.assertIn('observations: 1', '\n'.join(compare_policy(current,reference)))
        current['runs'][0]['branches'][0]['months'][0]['sites'][0]['population'] = 11
        with self.assertRaisesRegex(ValueError, 'Fixed control changed'):
            compare_policy(current,reference)

class PolicySuiteTests(unittest.TestCase):
    def fixture(self):
        from policy_suite import POLICIES
        sample = dict(month=13,sites=[dict(site=0,population=10,production_kg=1,
            ration_need_kg=[0,0,0,2],ration_eaten_kg=[0,0,0,1],tool_made_kg=1,
            production_probe=[1,1,5,10],labor_workers=[6,1,1,2])],
            economy_residuals=[0],source_residual=0,population_residual=0,
            ecology_relative_error=[0],ecology_water_relative_error=0)
        runs = [dict(seed=7307,tool_fraction=f,policy=p,config={},initial={},
            branches=[dict(closed=c,months=[sample]) for c in (False,True)])
            for f in (1,0) for p in POLICIES]
        return dict(complete=True,gpu='fixture',closure_months=1,recovery_months=0,runs=runs)

    def test_factorial_is_complete_and_policies_do_not_collapse(self):
        from policy_suite import render
        report = render(self.fixture())
        self.assertEqual(report.count('food-maintenance − food-only'),4)
        self.assertEqual(report.count('food-only − adaptive'),4)
        self.assertIn('16 monthly observations',report)
        self.assertIn('| none | 1/1 |',report)

    def test_missing_duplicate_and_unmatched_policy_fail_closed(self):
        from policy_suite import grouped
        data = self.fixture()
        data['runs'].pop()
        with self.assertRaisesRegex(ValueError, 'Incomplete'): grouped(data)
        data = self.fixture()
        data['runs'].append(data['runs'][0])
        with self.assertRaisesRegex(ValueError, 'Duplicate'): grouped(data)
        data = self.fixture()
        data['runs'][0]['initial'] = {'changed':True}
        with self.assertRaisesRegex(ValueError, 'Unmatched'): grouped(data)

if __name__ == '__main__':
    unittest.main()
