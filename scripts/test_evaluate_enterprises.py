import copy
import unittest
import tempfile
import json
from pathlib import Path
from evaluate_enterprises import summarize, write_report


def fixture():
    firm=dict(founded=12,closed=None,closing_reason=None,capital=10.,cash=11.,revenue=4.,
        wages=3.,rent=0.,dividends=0.,liquidation=0.,written_off=0.,paid_work=2.,completed_work=1.)
    return dict(complete=True,years=6,runs=[dict(seed=17,
        samples=[dict(month=m,population=100.,active_sites=2) for m in range(12,73,12)],
        enterprises=dict(firms=[firm]),shortage_site_years=2,abandonments=[],max_relative_residual=1e-6)])


class EmployerReports(unittest.TestCase):
    def test_survival_excludes_young_firms_and_accounts_reconcile(self):
        data=fixture()
        young=copy.deepcopy(data['runs'][0]['enterprises']['firms'][0]);young['founded']=60
        data['runs'][0]['enterprises']['firms'].append(young)
        row=summarize(data)
        self.assertEqual((row['founded'],row['five_year_eligible'],row['five_year_survivors']),(2,1,1))
        self.assertEqual(row['utilization'],.5)

    def test_closed_firm_is_not_a_survivor_and_liquidation_remains_accounted(self):
        data=fixture();firm=data['runs'][0]['enterprises']['firms'][0]
        firm.update(closed=24,closing_reason='working capital exhausted',cash=0.,liquidation=11.)
        row=summarize(data)
        self.assertEqual(row['five_year_survivors'],0)
        self.assertEqual(row['median_closed_lifetime_months'],12)
        self.assertEqual(row['closure_reasons'],{'working capital exhausted':1})

    def test_factorial_effects_require_all_four_matched_modes(self):
        rows=[]
        for mode, population in [('communal-equal',100),('communal-linked',95),
                                 ('operators-equal',102),('operators-linked',99)]:
            row=summarize(fixture());row.update(mode=mode,population=population);rows.append(row)
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)
            write_report(path,rows[:3]);self.assertEqual(json.loads((path/'effects.json').read_text()),[])
            write_report(path,rows)
            population=next(r for r in json.loads((path/'effects.json').read_text()) if r['metric']=='population')
            self.assertEqual(population['wages_without_operators'],-5)
            self.assertEqual(population['operators_with_equal_wages'],2)
            self.assertEqual(population['operators_with_linked_wages'],4)
            self.assertEqual(population['interaction'],2)

    def test_partial_trajectory_and_broken_ledger_rejected(self):
        for change in ('partial','ledger','residual'):
            data=fixture()
            if change=='partial': data['runs'][0]['samples'].pop()
            elif change=='ledger': data['runs'][0]['enterprises']['firms'][0]['cash']+=1
            else: data['runs'][0]['max_relative_residual']=.01
            with self.assertRaises(ValueError): summarize(data)


if __name__=='__main__': unittest.main()
