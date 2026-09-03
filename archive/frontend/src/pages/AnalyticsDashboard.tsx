import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  TrendingUp, TrendingDown, Users, Calendar, DollarSign, 
  BarChart3, PieChart, Activity, Download, RefreshCw,
  Eye, UserCheck, CreditCard, Target
} from 'lucide-react'
import { useAuth } from '../hooks/useAuth'
import { analyticsService } from '../services/analyticsService'

interface AnalyticsOverview {
  total_events: number
  published_events: number
  completed_events: number
  cancelled_events: number
  avg_registrations: number
  total_registrations: number
  estimated_revenue: number
}

interface EventTrend {
  month: string
  events_created: number
  total_registrations: number
  avg_fill_rate: number
}

interface TopEvent {
  id: string
  title: string
  current_registrations: number
  capacity_max: number
  fill_rate: number
  revenue: number
  status: string
}

interface CategoryPerformance {
  category_name: string
  event_count: number
  total_registrations: number
  avg_fill_rate: number
  total_revenue: number
}

interface RevenueData {
  monthlyRevenue: Array<{
    month: string
    revenue: number
    event_count: number
    total_registrations: number
  }>
  categoryRevenue: Array<{
    category: string
    revenue: number
    event_count: number
    avg_revenue_per_event: number
  }>
  tierRevenue: Array<{
    membership_tier: string
    registration_count: number
    total_revenue: number
  }>
}

interface EngagementData {
  engagement: Array<{
    membership_tier: string
    total_members: number
    active_members: number
    engagement_rate: number
    avg_events_per_member: number
  }>
  topMembers: Array<{
    first_name: string
    last_name: string
    membership_tier: string
    events_attended: number
    total_spent: number
  }>
  retention: Array<{
    cohort_month: string
    cohort_size: number
    active_this_month: number
    retention_rate: number
  }>
}

const AnalyticsDashboard = () => {
  const { user } = useAuth()
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [activeTab, setActiveTab] = useState<'overview' | 'revenue' | 'engagement'>('overview')
  
  // Data states
  const [overview, setOverview] = useState<AnalyticsOverview | null>(null)
  const [trends, setTrends] = useState<EventTrend[]>([])
  const [topEvents, setTopEvents] = useState<TopEvent[]>([])
  const [categoryPerformance, setCategoryPerformance] = useState<CategoryPerformance[]>([])
  const [revenueData, setRevenueData] = useState<RevenueData | null>(null)
  const [engagementData, setEngagementData] = useState<EngagementData | null>(null)

  useEffect(() => {
    fetchAnalyticsData()
  }, [])

  const fetchAnalyticsData = async () => {
    try {
      setLoading(true)
      
      // Fetch overview data
      const overviewResponse = await analyticsService.getEventsOverview()
      if (overviewResponse.success && overviewResponse.data) {
        // Map the actual API response to expected format
        const eventStats = overviewResponse.data.event_stats || {}
        const registrationStats = overviewResponse.data.registration_stats || {}
        const popularEvents = overviewResponse.data.popular_events || []
        
        // Create overview object from API response
        const overviewData: AnalyticsOverview = {
          total_events: eventStats.total_events || 0,
          published_events: eventStats.recent_events || 0,
          completed_events: eventStats.past_events || 0,
          cancelled_events: 0,
          avg_registrations: registrationStats.total_registrations || 0,
          total_registrations: registrationStats.total_registrations || 0,
          estimated_revenue: 0
        }
        
        setOverview(overviewData)
        setTrends([]) // Empty for now since API doesn't return trends
        setTopEvents(popularEvents.map((event: any) => ({
          id: event.id?.toString() || '',
          title: event.name || '',
          current_registrations: event.current_attendees || 0,
          capacity_max: event.capacity || 0,
          fill_rate: event.occupancy_rate || 0,
          revenue: 0,
          status: 'active'
        })))
        setCategoryPerformance([]) // Empty for now since API doesn't return category performance
      }

      // Fetch revenue data
      const revenueResponse = await analyticsService.getRevenueAnalytics()
      if (revenueResponse.success && revenueResponse.data) {
        setRevenueData(revenueResponse.data)
      }

      // Fetch engagement data
      const engagementResponse = await analyticsService.getMemberEngagement()
      if (engagementResponse.success && engagementResponse.data) {
        setEngagementData(engagementResponse.data)
      }
    } catch (error) {
      console.error('Failed to fetch analytics data:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleRefresh = async () => {
    setRefreshing(true)
    await fetchAnalyticsData()
    setRefreshing(false)
  }

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('zh-TW', {
      style: 'currency',
      currency: 'TWD',
      minimumFractionDigits: 0
    }).format(amount)
  }

  const formatPercentage = (value: number) => {
    return `${value.toFixed(1)}%`
  }

  const exportData = () => {
    // Implementation for data export
    console.log('Exporting analytics data...')
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center">
        <div className="luxury-glass p-8 rounded-2xl text-center">
          <div className="w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-luxury-platinum">載入分析數據中...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-luxury font-bold text-luxury-gold mb-2">
              分析儀表板
            </h1>
            <p className="text-luxury-platinum/80">
              平台營運數據與會員活動分析
            </p>
          </div>
          
          <div className="flex space-x-4">
            <button
              onClick={handleRefresh}
              disabled={refreshing}
              className="luxury-button-outline flex items-center gap-2"
            >
              <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
              {refreshing ? '更新中...' : '重新整理'}
            </button>
            
            <button
              onClick={exportData}
              className="luxury-button flex items-center gap-2"
            >
              <Download className="h-4 w-4" />
              匯出報告
            </button>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="flex space-x-1 mb-8">
          {[
            { key: 'overview', label: '總覽', icon: BarChart3 },
            { key: 'revenue', label: '營收分析', icon: DollarSign },
            { key: 'engagement', label: '會員參與', icon: Users }
          ].map(({ key, label, icon: Icon }) => (
            <button
              key={key}
              onClick={() => setActiveTab(key as any)}
              className={`flex items-center gap-2 px-6 py-3 rounded-lg transition-colors ${
                activeTab === key
                  ? 'bg-luxury-gold text-luxury-midnight-black font-medium'
                  : 'text-luxury-platinum hover:bg-luxury-gold/10'
              }`}
            >
              <Icon className="h-4 w-4" />
              {label}
            </button>
          ))}
        </div>

        {/* Overview Tab */}
        {activeTab === 'overview' && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="space-y-8"
          >
            {/* Key Metrics */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="luxury-glass p-6 rounded-2xl">
                <div className="flex items-center justify-between mb-4">
                  <div className="w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center">
                    <Calendar className="h-6 w-6 text-blue-400" />
                  </div>
                  <span className="text-luxury-platinum/60 text-sm">本月</span>
                </div>
                <div className="text-2xl font-bold text-luxury-gold mb-1">
                  {overview?.total_events || 0}
                </div>
                <div className="text-luxury-platinum/80 text-sm">總活動數</div>
                <div className="text-green-400 text-xs mt-2">
                  已發布: {overview?.published_events || 0}
                </div>
              </div>

              <div className="luxury-glass p-6 rounded-2xl">
                <div className="flex items-center justify-between mb-4">
                  <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center">
                    <Users className="h-6 w-6 text-green-400" />
                  </div>
                  <span className="text-luxury-platinum/60 text-sm">累計</span>
                </div>
                <div className="text-2xl font-bold text-luxury-gold mb-1">
                  {overview?.total_registrations || 0}
                </div>
                <div className="text-luxury-platinum/80 text-sm">總報名數</div>
                <div className="text-green-400 text-xs mt-2">
                  平均: {Math.round(overview?.avg_registrations || 0)} 人/活動
                </div>
              </div>

              <div className="luxury-glass p-6 rounded-2xl">
                <div className="flex items-center justify-between mb-4">
                  <div className="w-12 h-12 bg-luxury-gold/20 rounded-lg flex items-center justify-center">
                    <DollarSign className="h-6 w-6 text-luxury-gold" />
                  </div>
                  <span className="text-luxury-platinum/60 text-sm">預估</span>
                </div>
                <div className="text-2xl font-bold text-luxury-gold mb-1">
                  {formatCurrency(overview?.estimated_revenue || 0)}
                </div>
                <div className="text-luxury-platinum/80 text-sm">營收</div>
                <div className="text-green-400 text-xs mt-2">
                  本月預估
                </div>
              </div>

              <div className="luxury-glass p-6 rounded-2xl">
                <div className="flex items-center justify-between mb-4">
                  <div className="w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center">
                    <Activity className="h-6 w-6 text-purple-400" />
                  </div>
                  <span className="text-luxury-platinum/60 text-sm">狀態</span>
                </div>
                <div className="text-2xl font-bold text-luxury-gold mb-1">
                  {overview?.completed_events || 0}
                </div>
                <div className="text-luxury-platinum/80 text-sm">已完成</div>
                <div className="text-red-400 text-xs mt-2">
                  取消: {overview?.cancelled_events || 0}
                </div>
              </div>
            </div>

            {/* Monthly Trends */}
            <div className="luxury-glass p-6 rounded-2xl">
              <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                月度趨勢
              </h3>
              <div className="space-y-4">
                {(trends || []).slice(0, 6).map((trend, index) => (
                  <div key={trend.month} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                    <div className="flex items-center space-x-4">
                      <div className="text-luxury-platinum font-medium">
                        {new Date(trend.month + '-01').toLocaleDateString('zh-TW', { year: 'numeric', month: 'long' })}
                      </div>
                    </div>
                    <div className="flex items-center space-x-6 text-sm">
                      <div className="text-center">
                        <div className="text-luxury-gold font-medium">{trend.events_created}</div>
                        <div className="text-luxury-platinum/60">活動</div>
                      </div>
                      <div className="text-center">
                        <div className="text-luxury-gold font-medium">{trend.total_registrations}</div>
                        <div className="text-luxury-platinum/60">報名</div>
                      </div>
                      <div className="text-center">
                        <div className="text-luxury-gold font-medium">{formatPercentage(trend.avg_fill_rate)}</div>
                        <div className="text-luxury-platinum/60">滿座率</div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Top Performing Events */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <div className="luxury-glass p-6 rounded-2xl">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                  熱門活動
                </h3>
                <div className="space-y-4">
                  {(topEvents || []).slice(0, 5).map((event, index) => (
                    <div key={event.id} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                      <div className="flex items-center space-x-3">
                        <div className="w-8 h-8 bg-luxury-gold/20 rounded-full flex items-center justify-center text-luxury-gold font-medium text-sm">
                          {index + 1}
                        </div>
                        <div>
                          <div className="text-luxury-platinum font-medium text-sm">
                            {event.title}
                          </div>
                          <div className="text-luxury-platinum/60 text-xs">
                            {event.current_registrations}/{event.capacity_max} 人
                          </div>
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-luxury-gold font-medium text-sm">
                          {formatPercentage(event.fill_rate)}
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          {formatCurrency(event.revenue)}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div className="luxury-glass p-6 rounded-2xl">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                  類別表現
                </h3>
                <div className="space-y-4">
                  {(categoryPerformance || []).slice(0, 5).map((category, index) => (
                    <div key={category.category_name} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                      <div>
                        <div className="text-luxury-platinum font-medium text-sm">
                          {category.category_name}
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          {category.event_count} 個活動
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-luxury-gold font-medium text-sm">
                          {formatCurrency(category.total_revenue)}
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          {formatPercentage(category.avg_fill_rate)} 滿座率
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </motion.div>
        )}

        {/* Revenue Tab */}
        {activeTab === 'revenue' && revenueData && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="space-y-8"
          >
            {/* Monthly Revenue Chart */}
            <div className="luxury-glass p-6 rounded-2xl">
              <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                月度營收趨勢
              </h3>
              <div className="space-y-4">
                {(revenueData.monthlyRevenue || []).slice(0, 6).map((month) => (
                  <div key={month.month} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                    <div className="text-luxury-platinum font-medium">
                      {new Date(month.month + '-01').toLocaleDateString('zh-TW', { year: 'numeric', month: 'long' })}
                    </div>
                    <div className="flex items-center space-x-6 text-sm">
                      <div className="text-center">
                        <div className="text-luxury-gold font-medium">{formatCurrency(month.revenue)}</div>
                        <div className="text-luxury-platinum/60">營收</div>
                      </div>
                      <div className="text-center">
                        <div className="text-luxury-gold font-medium">{month.event_count}</div>
                        <div className="text-luxury-platinum/60">活動數</div>
                      </div>
                      <div className="text-center">
                        <div className="text-luxury-gold font-medium">{month.total_registrations}</div>
                        <div className="text-luxury-platinum/60">報名數</div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Category and Tier Revenue */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <div className="luxury-glass p-6 rounded-2xl">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                  類別營收
                </h3>
                <div className="space-y-4">
                  {(revenueData.categoryRevenue || []).map((category) => (
                    <div key={category.category} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                      <div>
                        <div className="text-luxury-platinum font-medium text-sm">
                          {category.category}
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          {category.event_count} 個活動
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-luxury-gold font-medium text-sm">
                          {formatCurrency(category.revenue)}
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          平均 {formatCurrency(category.avg_revenue_per_event)}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div className="luxury-glass p-6 rounded-2xl">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                  會員等級營收
                </h3>
                <div className="space-y-4">
                  {(revenueData.tierRevenue || []).map((tier) => (
                    <div key={tier.membership_tier} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                      <div className="flex items-center space-x-3">
                        <div className={`w-3 h-3 rounded-full ${
                          tier.membership_tier === 'Black Card' ? 'bg-luxury-gold' :
                          tier.membership_tier === 'Diamond' ? 'bg-blue-400' : 'bg-gray-400'
                        }`}></div>
                        <div>
                          <div className="text-luxury-platinum font-medium text-sm">
                            {tier.membership_tier}
                          </div>
                          <div className="text-luxury-platinum/60 text-xs">
                            {tier.registration_count} 次報名
                          </div>
                        </div>
                      </div>
                      <div className="text-luxury-gold font-medium text-sm">
                        {formatCurrency(tier.total_revenue)}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </motion.div>
        )}

        {/* Engagement Tab */}
        {activeTab === 'engagement' && engagementData && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="space-y-8"
          >
            {/* Engagement Metrics */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              {(engagementData.engagement || []).map((tier) => (
                <div key={tier.membership_tier} className="luxury-glass p-6 rounded-2xl">
                  <div className="flex items-center justify-between mb-4">
                    <h4 className="text-lg font-medium text-luxury-gold">
                      {tier.membership_tier}
                    </h4>
                    <div className={`w-3 h-3 rounded-full ${
                      tier.membership_tier === 'Black Card' ? 'bg-luxury-gold' :
                      tier.membership_tier === 'Diamond' ? 'bg-blue-400' : 'bg-gray-400'
                    }`}></div>
                  </div>
                  
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-luxury-platinum/80 text-sm">總會員</span>
                      <span className="text-luxury-platinum font-medium">{tier.total_members}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-luxury-platinum/80 text-sm">活躍會員</span>
                      <span className="text-luxury-platinum font-medium">{tier.active_members}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-luxury-platinum/80 text-sm">參與率</span>
                      <span className="text-luxury-gold font-medium">{formatPercentage(tier.engagement_rate)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-luxury-platinum/80 text-sm">平均活動數</span>
                      <span className="text-luxury-platinum font-medium">{tier.avg_events_per_member?.toFixed(1) || '0.0'}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Top Members and Retention */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <div className="luxury-glass p-6 rounded-2xl">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                  最活躍會員
                </h3>
                <div className="space-y-4">
                  {(engagementData.topMembers || []).slice(0, 8).map((member, index) => (
                    <div key={`${member.first_name}-${member.last_name}-${index}`} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                      <div className="flex items-center space-x-3">
                        <div className="w-8 h-8 bg-luxury-gold/20 rounded-full flex items-center justify-center text-luxury-gold font-medium text-sm">
                          {index + 1}
                        </div>
                        <div>
                          <div className="text-luxury-platinum font-medium text-sm">
                            {member.first_name} {member.last_name}
                          </div>
                          <div className="text-luxury-platinum/60 text-xs">
                            {member.membership_tier}
                          </div>
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-luxury-gold font-medium text-sm">
                          {member.events_attended} 活動
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          {formatCurrency(member.total_spent)}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div className="luxury-glass p-6 rounded-2xl">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
                  會員留存率
                </h3>
                <div className="space-y-4">
                  {(engagementData.retention || []).slice(0, 6).map((cohort) => (
                    <div key={cohort.cohort_month} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                      <div>
                        <div className="text-luxury-platinum font-medium text-sm">
                          {new Date(cohort.cohort_month + '-01').toLocaleDateString('zh-TW', { year: 'numeric', month: 'long' })}
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          {cohort.cohort_size} 新會員
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-luxury-gold font-medium text-sm">
                          {formatPercentage(cohort.retention_rate)}
                        </div>
                        <div className="text-luxury-platinum/60 text-xs">
                          {cohort.active_this_month} 活躍
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </div>
    </div>
  )
}

export default AnalyticsDashboard
