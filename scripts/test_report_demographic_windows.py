import unittest
from report_demographic_windows import summarize


class DemographicWindowTests(unittest.TestCase):
    def row(self, year):
        return dict(year=year, months=12, person_months=[120, 240, 60],
                    base_deaths=1, nutrition_deaths=2, illness_deaths=3,
                    potential_births=8, hunger_suppressed_births=1,
                    illness_suppressed_births=2, observed_births=5, observed_deaths=6,
                    food_need_kg=100, physical_shortfall_kg=10, access_shortfall_kg=20)

    def test_window_filters_and_reconciles_without_counting_flows_twice(self):
        h = dict(seed=17, demographic_audit=dict(years=[self.row(1), self.row(2)]))
        r = summarize(h, 1)
        self.assertEqual(r['observed_months'], 12)
        self.assertEqual(r['observed_deaths'], 6)
        self.assertEqual(r['birth_reconstruction_error'], 0)
        self.assertEqual(r['death_reconstruction_error'], 0)
        self.assertEqual(r['physical_deficit_share'], .1)
        self.assertEqual(r['access_deficit_share'], .2)
        self.assertEqual(summarize(h, 2)['observed_deaths'], 12)

    def test_missing_history_is_not_zero_mortality(self):
        with self.assertRaises(ValueError):
            summarize({}, 20)
        with self.assertRaises(ValueError):
            summarize(dict(demographic_audit=dict(years=[self.row(30)])), 20)
