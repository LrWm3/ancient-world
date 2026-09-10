"""Tests for diagnostic interpretation: annual transitions, denominators and incomplete runs."""
import unittest
import copy
from compare_integrated_history import compare
from analyze_integrated_history import summarize


def world():
    def site(holders, free=2):
        return {'site':0,'knowledge_holders':holders,'planned_land_freight':True,
                'free_land_freight_kg':free,'unrest':.8,'water_coverage':.5,'crowding':.2}
    def year(month, sites):
        return {'month':month,'integration':{'sites':sites},'population':100,'active_sites':len(sites),'cumulative_events':{}}
    return {'complete':True,'years':3,'runs':[{'seed':17,'samples':[
        year(12,[site([1,0],0)]),year(24,[site([0,1])]),year(36,[])],
        'shortage_site_years':1,'events':{},'abandonments':[{}],
        'max_relative_residual':1e-6,'seconds':1}]}


class IntegratedReport(unittest.TestCase):
    def test_annual_transitions_exclude_disappeared_towns(self):
        r=summarize(world())
        self.assertEqual((r['knowledge_losses'],r['knowledge_gains']), (1,1))
        self.assertEqual(r['site_years'],2)
        self.assertEqual(r['freight_saturated_site_years'],1)
        self.assertEqual(r['high_unrest_site_years'],2)
        self.assertEqual(r['final_single_holder_topics'],0)
        self.assertEqual(r['water_coverage'],0)

    def test_missing_and_duplicate_observation_rejected(self):
        for months in ([12,36], [12,12,36]):
            d=world()
            d['runs'][0]['samples']=d['runs'][0]['samples'][:len(months)]
            for sample, month in zip(d['runs'][0]['samples'],months): sample['month']=month
            with self.assertRaises(ValueError): summarize(d)
        d=world();d['complete']=False
        with self.assertRaises(ValueError): summarize(d)

    def test_policy_comparison_rejects_unexplained_early_divergence(self):
        a = world()
        b = copy.deepcopy(a)
        b['runs'][0]['samples'][1]['cumulative_events']['autonomy_negotiated'] = 1
        b['runs'][0]['samples'][2]['cumulative_events']['autonomy_negotiated'] = 1
        self.assertEqual(compare(a,b)['identical_pre_intervention_years'], 1)
        b['runs'][0]['samples'][0]['population'] = 99
        with self.assertRaises(ValueError): compare(a,b)
        self.assertEqual(compare(a,a)['identical_pre_intervention_years'], 3)


if __name__ == '__main__':
    unittest.main()
