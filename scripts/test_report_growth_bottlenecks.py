import copy
import unittest

from report_growth_bottlenecks import summarize
from test_report_monthly_food import fixture


class BottleneckTests(unittest.TestCase):
    def fixture(self):
        h = fixture()
        h['month'] = 4
        return h

    def test_local_deficits_and_transfers_are_separate(self):
        h = self.fixture()
        samples = h['demographic_audit']['food']['months']
        shortage = copy.deepcopy(samples[0])
        shortage.update(site=7, physically_available_kg=0)
        samples.append(shortage)
        a, b = summarize(h)['sites']
        self.assertEqual(a['access_shortfall_kg'], 5)
        self.assertEqual(b['physical_shortfall_kg'], 5)
        self.assertEqual(b['physical_shortage_months'], 1)
        self.assertEqual(a['physical_shortage_months'], 0)
        self.assertEqual(a['gpu_produced_kg'], 20)
        self.assertEqual(a['unclassified_net_kg'], 20)
        self.assertEqual(a['recorded_spoilage_kg'], 5)
        self.assertAlmostEqual(a['gpu_production_need_ratio'], 1/3)

    def test_gaps_and_missing_boundaries_are_not_zero_flow(self):
        h = self.fixture()
        samples = h['demographic_audit']['food']['months']
        late = copy.deepcopy(samples[0])
        late.update(month=4, opening=None)
        samples.append(late)
        row = summarize(h)['sites'][0]
        self.assertEqual(row['missing_months'], 1)
        self.assertEqual(row['observed_months'], 2)
        self.assertFalse(row['complete_boundaries'])
        self.assertIsNone(row['unclassified_net_kg'])
        self.assertIsNone(row['recorded_spoilage_kg'])

    def test_service_receipts_remain_dated_and_missing_is_unknown(self):
        h = self.fixture()
        self.assertIsNone(summarize(h)['upkeep_used'])
        h['culture'] = {'work_plans': [
            {'month': 1, 'upkeep': [{'requested': 100, 'granted': 100, 'used': 100}]},
            {'month': 3, 'upkeep': [{'requested': 2, 'granted': 1, 'used': .5}],
             'successor_expectation': {'actual_acquisition': True}}]}
        report = summarize(h)
        self.assertEqual(report['latest_plan_month'], 3)
        self.assertEqual(report['upkeep_requested'], 2)
        self.assertEqual(report['upkeep_used'], .5)
        self.assertEqual(report['successor_knowledge_acquisitions'], 1)


if __name__ == '__main__':
    unittest.main()
