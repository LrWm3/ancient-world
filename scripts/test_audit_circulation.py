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
