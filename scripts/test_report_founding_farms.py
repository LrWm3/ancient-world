import unittest

from report_founding_farms import summarize


def fixture():
    return dict(seed=17, month=24, economy_catalog=dict(
        agriculture=dict(crops=[dict(good='tuber', season=None)]),
        goods=[dict(id='tuber', food_energy=.3)]), sites=[dict(
            id=4, abandoned=False,
            stocks=dict(stock=[100, 42], ledger=[70], habitat=[1000, 160]),
            economy=dict(management=[1], crops=[[.08, 50, 2, 100]],
                         production_probe=[.9, 60, 5, 100], diagnostics=[3],
                         soil=[0, 20, 4], water=[10]))])


class FarmReportTests(unittest.TestCase):
    def test_mass_energy_and_edible_ledger_are_distinct(self):
        row = summarize(fixture())['sites'][0]
        crop = row['crops'][0]
        self.assertEqual(crop['cumulative_harvest_kg'], 100)
        self.assertEqual(crop['cumulative_nominal_harvest_food_kg'], 30)
        self.assertEqual(row['cumulative_edible_production_kg'], 70)
        self.assertEqual(crop['standing_kg'], 50)
        self.assertEqual(row['last_reported_limit'], 'water')

    def test_catalog_identity_and_archived_energy(self):
        h = fixture()
        h['economy_catalog']['goods'].insert(0, dict(id='other', food_energy=10))
        h['economy_catalog']['goods'][1]['food_energy'] = .2
        self.assertEqual(summarize(h)['sites'][0]['crops'][0]
                         ['cumulative_nominal_harvest_food_kg'], 20)

    def test_stale_probe_and_no_annual_inference(self):
        h = fixture()
        h['sites'][0]['abandoned'] = True
        row = summarize(h)['sites'][0]
        self.assertTrue(row['probe_may_be_stale'])
        self.assertNotIn('annual_water_limited_months', row)
        self.assertEqual(row['last_cultivated_ha'], 60)

    def test_rejects_legacy_and_mismatched_catalog(self):
        h = fixture()
        h['sites'][0]['economy']['management'][0] = 0
        with self.assertRaises(ValueError):
            summarize(h)
        h = fixture()
        h['economy_catalog']['agriculture']['crops'] = []
        with self.assertRaises(ValueError):
            summarize(h)


if __name__ == '__main__':
    unittest.main()
