import unittest
from audit_circulation import audit


class CirculationAuditTests(unittest.TestCase):
    def fixture(self):
        return {
            'seed': 17, 'month': 120,
            'sites': [{'id': 0, 'name': 'Port', 'abandoned': False,
                       'stocks': {'stock': [2, 10]}, 'economy': {'finance': [20, 100]}}],
            'society': {'households': [{'id': 0, 'site': 0, 'vacant_since': 100}],
                        'household_economy': {'accounts': [{'cash': 30, 'need': 18, 'hunger': 0}]},
                        'councils': [{'treasury': 10}], 'relocation': {'journeys': [{'cash': 4}]}},
            'people': [{'id': 0, 'died': None}],
            'participation': {'residents': {'0': {'person': 0, 'household': 0, 'presence': {'Traveling': 0}}}},
            'culture': {'institutions': [{'treasury': 5}]},
            'enterprises': {'firms': [{'id': 0, 'site': 0, 'closed': None, 'cash': 6,
                                     'revenue': 12, 'wages': 10, 'rent': 3, 'paid_work': 2, 'completed_work': 1}],
                            'orders': [{'escrow': 7}]},
            'export_contracts': [{'escrow': 8}], 'export_payments': [{'escrow': 9}],
            'expeditions': {'voyages': [{'purse': 3}]},
            'credit': {'issuance': {'receipts': [{'issued': 2}]}},
        }

    def test_wardrobe_spending_is_a_flow_not_extra_cash(self):
        h = self.fixture()
        before = audit(h)
        self.assertIsNone(before['household_clothing'])
        e = h['society']['household_economy']
        e['clothing_enabled'] = True
        e['accounts'][0]['wardrobe'] = dict(cloth_kg=2, purchased_kg=3, worn_kg=1, spending=9)
        after = audit(h)
        self.assertEqual(after['cash'], before['cash'])
        self.assertEqual(after['household_clothing']['spending'], 9)
        self.assertEqual(after['household_clothing']['cloth_kg'], 2)

    def test_food_request_observations_are_not_cash_or_delivered_food(self):
        h = self.fixture()
        before = audit(h)
        self.assertIsNone(before['food_requests'])
        h['trade_contact'] = {'food_requests': [dict(site=0, last_month=120,
            constraints=[1, 2, 3, 0, 0, 0, 0, 4, 5], requested_kg=1000,
            dispatched_kg=20, dispatches=4)]}
        after = audit(h)
        self.assertEqual(after['cash'], before['cash'])
        self.assertEqual(after['sites'], before['sites'])
        self.assertEqual(after['food_requests'][0]['constraints']['money'], 5)
        self.assertEqual(after['food_requests'][0]['dispatched_kg'], 20)

    def test_zero_allowance_does_not_imply_depletion(self):
        history = self.fixture()
        site = history['sites'][0]
        site['cell'] = 42
        site['economy']['reserves'] = [0, 0, 0, 0]
        source = {'mineral': 'lignite', 'ore_good': None,
                  'initial': [100, 200], 'remaining': [100, 150],
                  'extracted': [0, 50]}
        history['resources'] = {'sources': {'42': source}}
        def evidence():
            return audit(history)['sites'][0]['extraction']
        self.assertEqual(evidence()['source_status'], 'unsupported_for_metal_processing')
        self.assertEqual(evidence()['remaining_source_kg'], 100)
        source.update(mineral='hematite', ore_good=32)
        self.assertEqual(evidence()['source_status'], 'processable_stock_present')
        self.assertEqual(evidence()['ore_allowance_buffer_kg'], 0)
        source['remaining'][0] = 0
        source['extracted'][0] = 100
        self.assertEqual(evidence()['source_status'], 'empty_source')
        del history['resources']
        self.assertEqual(evidence()['source_status'], 'unregistered_or_unknown')

    def test_all_money_compartments_count_once(self):
        report = audit(self.fixture())
        self.assertEqual(report['cash_total'], 102)
        self.assertEqual(report['relative_money_residual'], 0)
        self.assertEqual(report['operators'][0]['operating_margin'], -1)
        self.assertEqual(report['operators'][0]['completion_per_paid_work'], .5)

    def test_lifetime_flows_are_not_cash_and_missing_is_unknown(self):
        history = self.fixture()
        report = audit(history)
        self.assertIsNone(report['household_cumulative_flows']['wages'])
        account = history['society']['household_economy']['accounts'][0]
        account.update(wages=1000, dividends=50, relief=20, food_spending=1040)
        report = audit(history)
        self.assertEqual(report['cash_total'], 102)
        self.assertEqual(report['household_cumulative_flows']['wages'], 1000)
        self.assertEqual(report['sites'][0]['household_cumulative_flows']['food_spending'], 1040)

    def test_wealth_tax_is_a_transfer_not_new_money(self):
        history = self.fixture()
        economy = history['society']['household_economy']
        economy['accounts'][0]['cash'] -= 12
        economy['accounts'][0]['wealth_tax_paid'] = 12
        history['society']['councils'][0]['treasury'] += 12
        economy['wealth_tax'] = {'enabled': True, 'receipts': [{'paid': 12}]}
        report = audit(history)
        self.assertEqual(report['cash_total'], 102)
        self.assertEqual(report['wealth_tax']['collected'], 12)
        self.assertEqual(report['wealth_tax']['assessments'], 1)

    def test_reclamation_is_a_transfer_not_new_money(self):
        history = self.fixture()
        economy = history['society']['household_economy']
        economy['accounts'][0]['cash'] -= 12
        history['sites'][0]['economy']['finance'][0] += 12
        economy['reclamation'] = {'receipts': [{'cash': 8}, {'cash': 4}]}
        report = audit(history)
        self.assertEqual(report['cash_total'], 102)
        self.assertEqual(report['relative_money_residual'], 0)
        self.assertEqual(report['reclaimed_cash'], 12)
        self.assertEqual(report['reclamation_cases'], 2)
        self.assertEqual(report['sites'][0]['vacant_household_cash'], 18)

    def test_vacancy_preserves_living_traveler_evidence(self):
        report = audit(self.fixture())
        retained = report['retained_households'][0]
        self.assertEqual(retained['known_living_members'], 1)
        self.assertEqual(retained['known_travelers'], 1)
        self.assertFalse(report['individual_demography'])
        self.assertEqual(report['sites'][0]['vacant_household_cash'], 30)

    def test_abandoned_nonvacant_wallet_and_dead_member(self):
        history = self.fixture()
        history['sites'][0]['abandoned'] = True
        history['society']['households'][0]['vacant_since'] = None
        history['people'][0]['died'] = 119
        report = audit(history)
        self.assertEqual(len(report['retained_households']), 1)
        self.assertEqual(report['retained_households'][0]['known_living_members'], 0)
        self.assertEqual(report['sites'][0]['vacant_household_cash'], 0)


if __name__ == '__main__':
    unittest.main()
