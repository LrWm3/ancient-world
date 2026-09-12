import copy
import unittest

from compare_food_access import compare, recent_interval


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.report = {
            'complete': True, 'observation_interval_months': 1,
            'years': 30, 'seeds': [17], 'common_share_override': 0.5,
            'runs': [{'seed': 17, 'samples': [{
                'year': 30, 'population': 100, 'active_sites': 1,
                'food_totals': [100, 90, 80, 80, 10, 10],
                'max_population_residual': 0, 'max_food_residual': 1e-8,
            }]}],
        }

    def test_known_partition_and_explicit_intervention(self):
        treatment = copy.deepcopy(self.report)
        treatment['common_share_override'] = 0.65
        with self.assertRaises(ValueError):
            compare(self.report, treatment, set())
        changed, rows = compare(self.report, treatment, {'common_share_override'})
        self.assertEqual(changed, {'common_share_override'})
        self.assertEqual(rows[0][1][:3], (100, 10, 10))

    def test_rejects_partial_missing_and_corrupt_results(self):
        for mutate in (
            lambda r: r.update(complete=False),
            lambda r: r.update(runs=[]),
            lambda r: r['runs'].append(copy.deepcopy(r['runs'][0])),
            lambda r: r['runs'][0]['samples'][0].update(year=20),
            lambda r: r['runs'][0]['samples'][0].update(population=float('nan')),
            lambda r: r['runs'][0]['samples'][0].update(food_totals=[100, 90, 80, 80, 10, 20]),
        ):
            trial = copy.deepcopy(self.report)
            mutate(trial)
            with self.assertRaises(ValueError):
                compare(self.report, trial, set())

    def test_recent_interval_exposes_late_decline_and_deprivation(self):
        report = copy.deepcopy(self.report)
        run = report['runs'][0]
        run['samples'].insert(0, {
            'year': 20, 'population': 150,
            'food_totals': [80, 75, 75, 75, 5, 0],
        })
        interval = recent_interval(report, run)
        self.assertEqual(interval['population_change'], -50)
        self.assertEqual(interval['physical_gap_pct'], 25)
        self.assertEqual(interval['access_gap_pct'], 50)
        self.assertEqual(interval['food_need'], 20)
        self.assertIsNone(recent_interval(self.report, self.report['runs'][0]))
        for mutate in (
            lambda r: r['samples'][0].update(year=30),
            lambda r: r['samples'][0].update(population=float('nan')),
            lambda r: r['samples'][0].update(food_totals=[101, 0, 0, 0, 0, 0]),
            lambda r: r['samples'][0].update(food_totals=[80, 75, 75, 75, 5, 1]),
        ):
            broken = copy.deepcopy(run)
            mutate(broken)
            with self.assertRaises(ValueError):
                recent_interval(report, broken)

    def test_no_recent_population_or_need_has_no_invented_percentage(self):
        report = copy.deepcopy(self.report)
        run = report['runs'][0]
        run['samples'][0]['population'] = 0
        before = copy.deepcopy(run['samples'][0])
        before['year'] = 20
        run['samples'].insert(0, before)
        interval = recent_interval(report, run)
        self.assertEqual(interval['population_change'], 0)
        self.assertIsNone(interval['population_change_pct'])
        self.assertIsNone(interval['access_gap_pct'])


if __name__ == '__main__':
    unittest.main()
