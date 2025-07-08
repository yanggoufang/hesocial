import React, { useState, useEffect } from 'react'
import { useAuth } from '../hooks/useAuth'
import { 
  TrendingUp, 
  Users, 
  Target,
  Activity,
  DollarSign,
  Phone,
  Mail,
  Calendar,
  Search,
  Filter,
  Plus,
  Edit,
  Eye,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  XCircle,
  CheckCircle,
  Star,
  Crown,
  Shield
} from 'lucide-react'

interface SalesLead {
  id: string
  firstName: string
  lastName: string
  email: string
  phone?: string
  company?: string
  position?: string
  leadScore: number
  annualIncome: number
  netWorth: number
  source: 'website' | 'referral' | 'event' | 'cold_call' | 'linkedin' | 'advertisement'
  status: 'new' | 'qualified' | 'contacted' | 'nurturing' | 'proposal' | 'negotiation' | 'closed_won' | 'closed_lost'
  assignedTo?: string
  interests: string[]
  lastContactDate?: string
  nextFollowUpDate?: string
  notes?: string
  createdAt: string
  updatedAt: string
}

interface SalesOpportunity {
  id: string
  leadId: string
  name: string
  description?: string
  stage: 'qualification' | 'needs_analysis' | 'proposal' | 'negotiation' | 'closed_won' | 'closed_lost'
  probability: number
  value: number
  membershipTier: 'Platinum' | 'Diamond' | 'Black Card'
  expectedCloseDate: string
  actualCloseDate?: string
  assignedTo?: string
  notes?: string
  createdAt: string
  updatedAt: string
  lead?: {
    firstName: string
    lastName: string
    email: string
  }
}

interface SalesMetrics {
  totalLeads: number
  qualifiedLeads: number
  totalOpportunities: number
  totalPipelineValue: number
  wonDeals: number
  conversionRate: number
  averageDealSize: number
  salesCycleLength: number
}

const SalesManagement: React.FC = () => {
  const { user } = useAuth()
  const [activeTab, setActiveTab] = useState<'leads' | 'opportunities' | 'metrics'>('leads')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  
  // Data
  const [leads, setLeads] = useState<SalesLead[]>([])
  const [opportunities, setOpportunities] = useState<SalesOpportunity[]>([])
  const [metrics, setMetrics] = useState<SalesMetrics | null>(null)
  
  // Pagination
  const [leadsPage, setLeadsPage] = useState(1)
  const [oppsPage, setOppsPage] = useState(1)
  const pageSize = 20
  
  // Filters
  const [leadFilters, setLeadFilters] = useState({
    search: '',
    status: '',
    source: '',
    assignedTo: ''
  })
  const [oppFilters, setOppFilters] = useState({
    search: '',
    stage: '',
    assignedTo: ''
  })

  useEffect(() => {
    if (activeTab === 'leads') {
      fetchLeads()
    } else if (activeTab === 'opportunities') {
      fetchOpportunities()
    } else if (activeTab === 'metrics') {
      fetchMetrics()
    }
  }, [activeTab, leadsPage, oppsPage, leadFilters, oppFilters])

  const fetchLeads = async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Mock API call - replace with actual API
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Mock data
      const mockLeads: SalesLead[] = [
        {
          id: '1',
          firstName: '張',
          lastName: '志明',
          email: 'zhang.zhiming@example.com',
          phone: '+886 912345678',
          company: '台積電',
          position: '副總經理',
          leadScore: 85,
          annualIncome: 25000000,
          netWorth: 150000000,
          source: 'linkedin',
          status: 'qualified',
          assignedTo: 'sales-001',
          interests: ['投資', '高爾夫', '藝術收藏'],
          lastContactDate: '2025-07-05',
          nextFollowUpDate: '2025-07-10',
          notes: '對私人銀行服務表現出高度興趣',
          createdAt: '2025-07-01T10:00:00Z',
          updatedAt: '2025-07-05T15:30:00Z'
        },
        {
          id: '2',
          firstName: '李',
          lastName: '美華',
          email: 'li.meihua@example.com',
          phone: '+886 987654321',
          company: '鴻海精密',
          position: '財務長',
          leadScore: 92,
          annualIncome: 30000000,
          netWorth: 200000000,
          source: 'referral',
          status: 'nurturing',
          assignedTo: 'sales-002',
          interests: ['遊艇', '紅酒', '慈善'],
          lastContactDate: '2025-07-06',
          nextFollowUpDate: '2025-07-12',
          notes: '透過現有客戶推薦，對 Black Card 會員感興趣',
          createdAt: '2025-07-02T14:00:00Z',
          updatedAt: '2025-07-06T09:15:00Z'
        }
      ]
      
      setLeads(mockLeads)
    } catch (err) {
      setError('Failed to fetch leads')
    } finally {
      setLoading(false)
    }
  }

  const fetchOpportunities = async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      const mockOpportunities: SalesOpportunity[] = [
        {
          id: '1',
          leadId: '1',
          name: 'Black Card 會員申請 - 張志明',
          description: '高淨值客戶申請 Black Card 會員資格',
          stage: 'proposal',
          probability: 75,
          value: 500000,
          membershipTier: 'Black Card',
          expectedCloseDate: '2025-07-20',
          assignedTo: 'sales-001',
          notes: '已提交初步提案，等待客戶回覆',
          createdAt: '2025-07-03T10:00:00Z',
          updatedAt: '2025-07-06T16:00:00Z',
          lead: {
            firstName: '張',
            lastName: '志明',
            email: 'zhang.zhiming@example.com'
          }
        },
        {
          id: '2',
          leadId: '2',
          name: 'Diamond 會員升級 - 李美華',
          description: '現有 Platinum 會員申請升級至 Diamond',
          stage: 'negotiation',
          probability: 85,
          value: 300000,
          membershipTier: 'Diamond',
          expectedCloseDate: '2025-07-15',
          assignedTo: 'sales-002',
          notes: '價格談判中，客戶對服務內容滿意',
          createdAt: '2025-07-04T09:00:00Z',
          updatedAt: '2025-07-07T11:30:00Z',
          lead: {
            firstName: '李',
            lastName: '美華',
            email: 'li.meihua@example.com'
          }
        }
      ]
      
      setOpportunities(mockOpportunities)
    } catch (err) {
      setError('Failed to fetch opportunities')
    } finally {
      setLoading(false)
    }
  }

  const fetchMetrics = async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Mock API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      const mockMetrics: SalesMetrics = {
        totalLeads: 156,
        qualifiedLeads: 89,
        totalOpportunities: 34,
        totalPipelineValue: 12500000,
        wonDeals: 18,
        conversionRate: 57.1,
        averageDealSize: 425000,
        salesCycleLength: 45
      }
      
      setMetrics(mockMetrics)
    } catch (err) {
      setError('Failed to fetch metrics')
    } finally {
      setLoading(false)
    }
  }

  const getLeadStatusBadge = (status: string) => {
    const styles = {
      'new': 'bg-blue-100 text-blue-800',
      'qualified': 'bg-green-100 text-green-800',
      'contacted': 'bg-yellow-100 text-yellow-800',
      'nurturing': 'bg-purple-100 text-purple-800',
      'proposal': 'bg-orange-100 text-orange-800',
      'negotiation': 'bg-indigo-100 text-indigo-800',
      'closed_won': 'bg-emerald-100 text-emerald-800',
      'closed_lost': 'bg-red-100 text-red-800'
    }

    const labels = {
      'new': '新線索',
      'qualified': '已審核',
      'contacted': '已聯繫',
      'nurturing': '培養中',
      'proposal': '提案階段',
      'negotiation': '談判中',
      'closed_won': '成交',
      'closed_lost': '失單'
    }

    return (
      <span className={`inline-flex px-2 py-1 rounded-full text-xs font-medium ${styles[status as keyof typeof styles]}`}>
        {labels[status as keyof typeof labels]}
      </span>
    )
  }

  const getOpportunityStage = (stage: string) => {
    const labels = {
      'qualification': '資格審核',
      'needs_analysis': '需求分析',
      'proposal': '提案階段',
      'negotiation': '談判中',
      'closed_won': '成交',
      'closed_lost': '失單'
    }
    return labels[stage as keyof typeof labels] || stage
  }

  const getMembershipTierBadge = (tier: string) => {
    const styles = {
      'Platinum': 'bg-gray-100 text-gray-800 border-gray-300',
      'Diamond': 'bg-blue-100 text-blue-800 border-blue-300',
      'Black Card': 'bg-black text-white border-black'
    }
    
    const icons = {
      'Platinum': <Star className="w-3 h-3" />,
      'Diamond': <Crown className="w-3 h-3" />,
      'Black Card': <Shield className="w-3 h-3" />
    }

    return (
      <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border ${styles[tier as keyof typeof styles]}`}>
        {icons[tier as keyof typeof icons]}
        {tier}
      </span>
    )
  }

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('zh-TW', { 
      style: 'currency', 
      currency: 'TWD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0
    }).format(amount)
  }

  if (!user || (user.role !== 'admin' && user.role !== 'super_admin')) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <Shield className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h1 className="text-2xl font-bold text-gray-900 mb-2">存取被拒</h1>
          <p className="text-gray-600">您沒有權限存取銷售管理系統。</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-3">
                <TrendingUp className="w-8 h-8 text-purple-600" />
                銷售管理系統
              </h1>
              <p className="text-gray-600 mt-2">管理銷售線索、商機與績效分析</p>
            </div>
            <button
              onClick={() => {
                if (activeTab === 'leads') fetchLeads()
                else if (activeTab === 'opportunities') fetchOpportunities()
                else fetchMetrics()
              }}
              className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              重新整理
            </button>
          </div>
        </div>

        {/* Error Message */}
        {error && (
          <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4">
            <div className="flex items-center">
              <XCircle className="w-5 h-5 text-red-600 mr-2" />
              <p className="text-red-800">{error}</p>
              <button
                onClick={() => setError(null)}
                className="ml-auto text-red-600 hover:text-red-800"
              >
                <XCircle className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Tabs */}
        <div className="mb-6">
          <div className="border-b border-gray-200">
            <nav className="-mb-px flex space-x-8">
              <button
                onClick={() => setActiveTab('leads')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'leads'
                    ? 'border-purple-500 text-purple-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Users className="w-4 h-4" />
                  銷售線索
                </div>
              </button>
              <button
                onClick={() => setActiveTab('opportunities')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'opportunities'
                    ? 'border-purple-500 text-purple-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Target className="w-4 h-4" />
                  銷售商機
                </div>
              </button>
              <button
                onClick={() => setActiveTab('metrics')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'metrics'
                    ? 'border-purple-500 text-purple-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Activity className="w-4 h-4" />
                  績效分析
                </div>
              </button>
            </nav>
          </div>
        </div>

        {/* Content */}
        {activeTab === 'leads' && (
          <div>
            {/* Leads Search & Filters */}
            <div className="bg-white rounded-lg shadow-sm border mb-6 p-4">
              <div className="flex items-center gap-4 mb-4">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
                  <input
                    type="text"
                    placeholder="搜尋線索..."
                    value={leadFilters.search}
                    onChange={(e) => setLeadFilters(prev => ({ ...prev, search: e.target.value }))}
                    className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  />
                </div>
                <select
                  value={leadFilters.status}
                  onChange={(e) => setLeadFilters(prev => ({ ...prev, status: e.target.value }))}
                  className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                >
                  <option value="">所有狀態</option>
                  <option value="new">新線索</option>
                  <option value="qualified">已審核</option>
                  <option value="contacted">已聯繫</option>
                  <option value="nurturing">培養中</option>
                </select>
                <button className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700">
                  <Plus className="w-4 h-4" />
                  新增線索
                </button>
              </div>
            </div>

            {/* Leads Table */}
            <div className="bg-white rounded-lg shadow-sm border overflow-hidden">
              {loading ? (
                <div className="flex items-center justify-center py-12">
                  <RefreshCw className="w-6 h-6 text-gray-400 animate-spin" />
                  <span className="ml-2 text-gray-600">載入中...</span>
                </div>
              ) : (
                <div className="overflow-x-auto">
                  <table className="w-full">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">線索資訊</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">分數</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">狀態</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">財務資訊</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">下次跟進</th>
                        <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">操作</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {leads.map((lead) => (
                        <tr key={lead.id} className="hover:bg-gray-50">
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div>
                              <div className="text-sm font-medium text-gray-900">{lead.firstName} {lead.lastName}</div>
                              <div className="text-sm text-gray-500">{lead.email}</div>
                              <div className="text-xs text-gray-400">{lead.company} - {lead.position}</div>
                            </div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div className="flex items-center">
                              <div className="text-sm font-medium text-gray-900">{lead.leadScore}</div>
                              <div className="ml-2 w-16 bg-gray-200 rounded-full h-2">
                                <div 
                                  className="bg-purple-600 h-2 rounded-full" 
                                  style={{ width: `${lead.leadScore}%` }}
                                ></div>
                              </div>
                            </div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            {getLeadStatusBadge(lead.status)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            <div>年收: {formatCurrency(lead.annualIncome)}</div>
                            <div className="text-gray-500">淨值: {formatCurrency(lead.netWorth)}</div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {lead.nextFollowUpDate ? new Date(lead.nextFollowUpDate).toLocaleDateString() : '-'}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                            <div className="flex items-center justify-end gap-2">
                              <button className="text-gray-600 hover:text-gray-900">
                                <Eye className="w-4 h-4" />
                              </button>
                              <button className="text-blue-600 hover:text-blue-900">
                                <Edit className="w-4 h-4" />
                              </button>
                              <button className="text-green-600 hover:text-green-900">
                                <Phone className="w-4 h-4" />
                              </button>
                              <button className="text-purple-600 hover:text-purple-900">
                                <Mail className="w-4 h-4" />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'opportunities' && (
          <div>
            {/* Opportunities Filters */}
            <div className="bg-white rounded-lg shadow-sm border mb-6 p-4">
              <div className="flex items-center gap-4">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
                  <input
                    type="text"
                    placeholder="搜尋商機..."
                    value={oppFilters.search}
                    onChange={(e) => setOppFilters(prev => ({ ...prev, search: e.target.value }))}
                    className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  />
                </div>
                <select
                  value={oppFilters.stage}
                  onChange={(e) => setOppFilters(prev => ({ ...prev, stage: e.target.value }))}
                  className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                >
                  <option value="">所有階段</option>
                  <option value="qualification">資格審核</option>
                  <option value="needs_analysis">需求分析</option>
                  <option value="proposal">提案階段</option>
                  <option value="negotiation">談判中</option>
                </select>
                <button className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700">
                  <Plus className="w-4 h-4" />
                  新增商機
                </button>
              </div>
            </div>

            {/* Opportunities Table */}
            <div className="bg-white rounded-lg shadow-sm border overflow-hidden">
              {loading ? (
                <div className="flex items-center justify-center py-12">
                  <RefreshCw className="w-6 h-6 text-gray-400 animate-spin" />
                  <span className="ml-2 text-gray-600">載入中...</span>
                </div>
              ) : (
                <div className="overflow-x-auto">
                  <table className="w-full">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">商機名稱</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">階段</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">會員等級</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">機率</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">價值</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">預期成交</th>
                        <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">操作</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {opportunities.map((opp) => (
                        <tr key={opp.id} className="hover:bg-gray-50">
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div>
                              <div className="text-sm font-medium text-gray-900">{opp.name}</div>
                              <div className="text-sm text-gray-500">{opp.lead?.firstName} {opp.lead?.lastName}</div>
                              <div className="text-xs text-gray-400">{opp.lead?.email}</div>
                            </div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {getOpportunityStage(opp.stage)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            {getMembershipTierBadge(opp.membershipTier)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div className="flex items-center">
                              <div className="text-sm font-medium text-gray-900">{opp.probability}%</div>
                              <div className="ml-2 w-16 bg-gray-200 rounded-full h-2">
                                <div 
                                  className="bg-green-500 h-2 rounded-full" 
                                  style={{ width: `${opp.probability}%` }}
                                ></div>
                              </div>
                            </div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                            {formatCurrency(opp.value)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {new Date(opp.expectedCloseDate).toLocaleDateString()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                            <div className="flex items-center justify-end gap-2">
                              <button className="text-gray-600 hover:text-gray-900">
                                <Eye className="w-4 h-4" />
                              </button>
                              <button className="text-blue-600 hover:text-blue-900">
                                <Edit className="w-4 h-4" />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'metrics' && metrics && (
          <div>
            {/* Metrics Cards */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">總線索數</p>
                    <p className="text-3xl font-bold text-gray-900">{metrics.totalLeads}</p>
                  </div>
                  <Users className="w-8 h-8 text-blue-600" />
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">合格線索</p>
                    <p className="text-3xl font-bold text-gray-900">{metrics.qualifiedLeads}</p>
                  </div>
                  <CheckCircle className="w-8 h-8 text-green-600" />
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">總商機數</p>
                    <p className="text-3xl font-bold text-gray-900">{metrics.totalOpportunities}</p>
                  </div>
                  <Target className="w-8 h-8 text-purple-600" />
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">管道總值</p>
                    <p className="text-3xl font-bold text-gray-900">{formatCurrency(metrics.totalPipelineValue)}</p>
                  </div>
                  <DollarSign className="w-8 h-8 text-yellow-600" />
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">成交數</p>
                    <p className="text-3xl font-bold text-gray-900">{metrics.wonDeals}</p>
                  </div>
                  <CheckCircle className="w-8 h-8 text-emerald-600" />
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">轉換率</p>
                    <p className="text-3xl font-bold text-gray-900">{metrics.conversionRate}%</p>
                  </div>
                  <TrendingUp className="w-8 h-8 text-indigo-600" />
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">平均成交金額</p>
                    <p className="text-3xl font-bold text-gray-900">{formatCurrency(metrics.averageDealSize)}</p>
                  </div>
                  <DollarSign className="w-8 h-8 text-orange-600" />
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">銷售週期</p>
                    <p className="text-3xl font-bold text-gray-900">{metrics.salesCycleLength} 天</p>
                  </div>
                  <Calendar className="w-8 h-8 text-red-600" />
                </div>
              </div>
            </div>

            {/* Additional Analytics */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <h3 className="text-lg font-medium text-gray-900 mb-4">銷售漏斗</h3>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">新線索</span>
                    <span className="text-sm font-medium">67</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">合格線索</span>
                    <span className="text-sm font-medium">89</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">提案階段</span>
                    <span className="text-sm font-medium">34</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">談判中</span>
                    <span className="text-sm font-medium">18</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">成交</span>
                    <span className="text-sm font-medium">18</span>
                  </div>
                </div>
              </div>
              
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <h3 className="text-lg font-medium text-gray-900 mb-4">會員等級分布</h3>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Star className="w-4 h-4 text-gray-600" />
                      <span className="text-sm text-gray-600">Platinum</span>
                    </div>
                    <span className="text-sm font-medium">12</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Crown className="w-4 h-4 text-blue-600" />
                      <span className="text-sm text-gray-600">Diamond</span>
                    </div>
                    <span className="text-sm font-medium">15</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Shield className="w-4 h-4 text-black" />
                      <span className="text-sm text-gray-600">Black Card</span>
                    </div>
                    <span className="text-sm font-medium">7</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default SalesManagement