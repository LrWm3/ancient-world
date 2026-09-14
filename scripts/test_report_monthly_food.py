import unittest

from report_monthly_food import flow, rows


def boundary(food, production=0, consumption=0, spoilage=0):
    return dict(food_kg=food, ledger_kg=[production, consumption, spoilage])


def fixture():
    return dict(seed=17, demographic_audit=dict(food=dict(months=[dict(
        site=3, month=2, opening=boundary(100), before_gpu=boundary(130),
        after_gpu=boundary(90, 20, 55, 5), closing=boundary(80, 20, 55, 5),
        need_kg=60, eaten_kg=55, physically_available_kg=150,
        cultivated_ha=10, tool_multiplier=1, limiting_resource=0,
        crop_harvest_kg=[0]*6)])))


class MonthlyFoodTests(unittest.TestCase):
    def test_import_production_consumption_spoilage_export(self):
        row = list(rows(fixture()))[0]
        self.assertEqual(row['before_production']['unclassified_net_kg'], 30)
        self.assertEqual(row['gpu']['unclassified_net_kg'], 0)
        self.assertEqual(row['after_production']['unclassified_net_kg'], -10)
        self.assertEqual(row['whole_month']['unclassified_net_kg'], 20)
        self.assertEqual(row['access_shortfall_kg'], 5)
        self.assertEqual(row['physical_shortfall_kg'], 0)

    def test_physical_shortage_is_capped_by_unmet(self):
        h = fixture()
        h['demographic_audit']['food']['months'][0]['physically_available_kg'] = 1
        row = list(rows(h))[0]
        self.assertEqual(row['physical_shortfall_kg'], 5)
        self.assertEqual(row['access_shortfall_kg'], 0)

    def test_missing_and_duplicate_observations_rejected(self):
        with self.assertRaises(ValueError):
            list(rows({}))
        h = fixture()
        h['demographic_audit']['food']['months'] *= 2
        with self.assertRaises(ValueError):
            list(rows(h))
        self.assertIsNone(flow(None, boundary(20)))

    def test_balance_remainder_does_not_hide_missing_flow(self):
        observed = flow(boundary(100), boundary(70, 0, 20, 0))
        self.assertEqual(observed['unclassified_net_kg'], -10)


if __name__ == '__main__':
    unittest.main()
