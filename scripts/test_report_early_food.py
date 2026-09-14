import unittest
from report_early_food import summarize


class EarlyFoodTests(unittest.TestCase):
    def fixture(self):
        return dict(seed=17, month=12, initial_food=100,
                    sites=[dict(id=0, economy=dict(management=[1], crops=[[0, 0, 0, 9999]]),
                                stocks=dict(stock=[10, 40], ledger=[80, 120, 20]),
                                lifecycle=dict(timeline=dict(samples=[dict(food=0), dict(food=40)])))])

    def test_raw_crop_mass_is_not_added_to_food_energy(self):
        r = summarize(self.fixture())
        self.assertEqual(r['food_produced_kg'], 80)
        self.assertEqual(r['initial_food_kg'] + r['food_produced_kg'],
                         r['food_kg'] + r['food_eaten_kg'] + r['food_spoiled_kg'])
        self.assertEqual(r['sites'][0]['empty_food_sample_months'], 1)
        self.assertIsNone(r['food_dispatched_kg'])

    def test_requests_are_not_delivery_and_legacy_growth_is_not_harvest(self):
        h = self.fixture()
        h['trade_contact'] = dict(food_requests=[dict(requested_kg=10000, dispatched_kg=2, dispatches=1)])
        self.assertEqual(summarize(h)['food_dispatched_kg'], 2)
        h['sites'][0]['economy']['management'][0] = 0
        with self.assertRaises(ValueError):
            summarize(h)
