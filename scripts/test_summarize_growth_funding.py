import unittest

from summarize_growth_funding import operational, summarize


class FundingSummaryTests(unittest.TestCase):
    def fixture(self):
        return {
            'seed': 17, 'month': 240,
            'sites': [{'stocks': {'stock': [100]}, 'abandoned': False,
                       'economy': {'finance': [30]}}],
            'society': {
                'household_economy': {'accounts': [
                    {'cash': 50, 'wealth_tax_paid': 4, 'food_spending': 9}]},
                'councils': [{'treasury': 20}],
                'council_funding': {'emergency_town_support': {'requested': 8, 'paid': 3}}},
            'culture': {'institutions': [
                {'active': True, 'treasury': 0, 'dues': 5, 'capacity': {'readiness': 0}},
                {'active': True, 'treasury': 2, 'dues': 8, 'capacity': {'readiness': .5}},
                {'active': False, 'treasury': 1, 'dues': 2, 'capacity': {'readiness': 1}}]},
            'demographic_audit': {'years': [
                {'year': year, 'months': 12, 'observed_births': 3, 'observed_deaths': 2,
                 'food_need_kg': need, 'physical_shortfall_kg': physical,
                 'access_shortfall_kg': access}
                for year, need, physical, access in [(10, 100, 10, 0), (20, 900, 0, 90)]]}}

    def test_weighted_gaps_coverage_and_readiness(self):
        result = summarize(self.fixture())
        self.assertEqual(result['physical_gap_pct'], 1)
        self.assertEqual(result['access_gap_pct'], 9)
        self.assertEqual(result['audit_months'], 24)
        self.assertEqual(result['final_decade_months'], 12)
        self.assertEqual(result['final_decade_births'], 3)
        self.assertEqual(result['active_institutions'], 2)
        self.assertEqual(result['operational_institutions'], 1)
        self.assertEqual(result['institution_cash'], 3)
        self.assertEqual(result['household_cash'], 50)

    def test_readiness_alone_does_not_establish_operation(self):
        institution = {'active': True, 'capacity': {
            'readiness': 1, 'mandate': {'holder': None}, 'building': None}}
        self.assertFalse(operational(institution))
        institution['capacity']['mandate']['holder'] = 4
        self.assertTrue(operational(institution))
        institution['capacity']['building'] = {
            'construction_remaining': 1, 'condition': 1, 'facility': None}
        self.assertFalse(operational(institution))
        institution['capacity']['building']['construction_remaining'] = 0
        self.assertTrue(operational(institution))
        institution['capacity']['building']['condition'] = .1
        self.assertFalse(operational(institution))

    def test_absent_audit_is_not_reported_as_zero_shortage(self):
        history = self.fixture()
        history['demographic_audit']['years'] = []
        with self.assertRaisesRegex(ValueError, 'no retained annual'):
            summarize(history)


if __name__ == '__main__':
    unittest.main()
