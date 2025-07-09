import React, { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import registrationService, { Registration } from '../services/registrationService';
import {
  Calendar,
  MapPin,
  Clock,
  Eye,
  Edit,
  X,
  Check,
  AlertTriangle,
  CreditCard,
  Filter,
  Search,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  ExternalLink,
  Loader2,
  Ticket,
} from 'lucide-react';

const MyRegistrations: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const { isAuthenticated } = useAuth();

  const [registrations, setRegistrations] = useState<Registration[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const [filters, setFilters] = useState({
    status: '',
    paymentStatus: '',
    search: '',
  });
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [totalRegistrations, setTotalRegistrations] = useState(0);
  const pageSize = 10;

  const [selectedRegistration, setSelectedRegistration] = useState<Registration | null>(null);
  const [showEditModal, setShowEditModal] = useState(false);
  const [editRequests, setEditRequests] = useState('');
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login');
      return;
    }

    if (location.state?.message) {
      setSuccessMessage(location.state.message);
      window.history.replaceState({}, document.title);
    }

    fetchRegistrations();
  }, [isAuthenticated, navigate, currentPage, filters]);

  const fetchRegistrations = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await registrationService.getUserRegistrations({
        ...filters,
        page: currentPage,
        limit: pageSize,
      });

      if (response.success && response.data) {
        setRegistrations(response.data);
        if (response.pagination) {
          setTotalPages(response.pagination.totalPages);
          setTotalRegistrations(response.pagination.total);
        }
      } else {
        setError(response.error || '無法獲取活動報名記錄');
      }
    } catch (err) {
      setError('發生網絡錯誤，請稍後再試');
    } finally {
      setLoading(false);
    }
  };

  const handleFilterChange = (key: string, value: string) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
    setCurrentPage(1);
  };

  const clearFilters = () => {
    setFilters({
      status: '',
      paymentStatus: '',
      search: '',
    });
    setCurrentPage(1);
  };

  const handleEditRegistration = async () => {
    if (!selectedRegistration) return;
    setActionLoading('edit');
    try {
      const response = await registrationService.updateRegistration(selectedRegistration.id, {
        specialRequests: editRequests,
      });
      if (response.success) {
        setShowEditModal(false);
        fetchRegistrations();
        setSuccessMessage('報名資訊已成功更新');
      } else {
        setError(response.error || '更新報名資訊失敗');
      }
    } catch (err) {
      setError('發生網絡錯誤');
    } finally {
      setActionLoading(null);
    }
  };

  const handleCancelRegistration = async (registrationId: string) => {
    if (window.confirm('您確定要取消此次活動報名嗎？此操作無法復原。')) {
      setActionLoading(`cancel-${registrationId}`);
      try {
        const response = await registrationService.cancelRegistration(registrationId);
        if (response.success) {
          fetchRegistrations();
          setSuccessMessage('活動報名已成功取消');
        } else {
          setError(response.error || '取消活動報名失敗');
        }
      } catch (err) {
        setError('發生網絡錯誤');
      } finally {
        setActionLoading(null);
      }
    }
  };

  const openEditModal = (registration: Registration) => {
    setSelectedRegistration(registration);
    setEditRequests(registration.specialRequests || '');
    setShowEditModal(true);
  };

  const getStatusBadge = (status: string) => {
    const statusMap: { [key: string]: { text: string; styles: string; icon: React.ReactNode } } = {
      pending: { text: '審核中', styles: 'bg-yellow-900/50 text-yellow-300 border-yellow-700', icon: <Clock className="w-3 h-3" /> },
      approved: { text: '已核准', styles: 'bg-green-900/50 text-green-300 border-green-700', icon: <Check className="w-3 h-3" /> },
      rejected: { text: '已婉拒', styles: 'bg-red-900/50 text-red-300 border-red-700', icon: <X className="w-3 h-3" /> },
      cancelled: { text: '已取消', styles: 'bg-gray-700/50 text-gray-300 border-gray-500', icon: <AlertTriangle className="w-3 h-3" /> },
    };
    const { text, styles, icon } = statusMap[status] || { text: status, styles: '', icon: null };
    return (
      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border ${styles}`}>
        {icon}
        {text}
      </span>
    );
  };

  const getPaymentBadge = (status: string) => {
    const statusMap: { [key: string]: { text: string; styles: string; icon: React.ReactNode } } = {
      pending: { text: '待付款', styles: 'bg-yellow-900/50 text-yellow-300', icon: <CreditCard className="w-3 h-3" /> },
      paid: { text: '已付款', styles: 'bg-green-900/50 text-green-300', icon: <CreditCard className="w-3 h-3" /> },
      refunded: { text: '已退款', styles: 'bg-blue-900/50 text-blue-300', icon: <CreditCard className="w-3 h-3" /> },
    };
    const { text, styles, icon } = statusMap[status] || { text: status, styles: '', icon: null };
    return (
      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium ${styles}`}>
        {icon}
        {text}
      </span>
    );
  };

  const formatDateTime = (dateString: string) => {
    return new Date(dateString).toLocaleString('zh-TW', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    });
  };

  const canEdit = (registration: Registration) => {
    return registration.status === 'pending' && new Date(registration.eventDateTime) > new Date();
  };

  const canCancel = (registration: Registration) => {
    const hoursUntilEvent = (new Date(registration.eventDateTime).getTime() - new Date().getTime()) / (1000 * 3600);
    return registration.status !== 'cancelled' && registration.status !== 'rejected' && hoursUntilEvent > 24;
  };

  if (!isAuthenticated) return null;

  return (
    <div className="min-h-screen bg-luxury-midnight-black text-luxury-platinum font-sans">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="mb-10">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-4xl font-luxury font-bold text-luxury-gold tracking-wider">我的活動報名</h1>
              <p className="text-luxury-platinum/80 mt-3 text-lg">管理您的活動報名與申請狀態</p>
            </div>
            <button
              onClick={() => navigate('/events')}
              className="luxury-button-outline inline-flex items-center gap-2"
            >
              <ExternalLink className="w-4 h-4" />
              探索更多活動
            </button>
          </div>
        </div>

        {successMessage && (
          <div className="mb-6 bg-green-900/30 border border-green-600/50 rounded-lg p-4 flex items-center justify-between backdrop-blur-sm">
            <div className="flex items-center gap-3">
              <Check className="w-5 h-5 text-green-400" />
              <p className="text-green-200">{successMessage}</p>
            </div>
            <button onClick={() => setSuccessMessage(null)} className="text-green-400 hover:text-green-200">
              <X className="w-4 h-4" />
            </button>
          </div>
        )}

        {error && (
          <div className="mb-6 bg-red-900/30 border border-red-600/50 rounded-lg p-4 flex items-center justify-between backdrop-blur-sm">
            <div className="flex items-center gap-3">
              <AlertTriangle className="w-5 h-5 text-red-400" />
              <p className="text-red-200">{error}</p>
            </div>
            <button onClick={() => setError(null)} className="text-red-400 hover:text-red-200">
              <X className="w-4 h-4" />
            </button>
          </div>
        )}

        <div className="luxury-glass rounded-xl shadow-2xl mb-8 p-6">
          <div className="flex items-center justify-between mb-5">
            <h3 className="text-xl font-luxury font-semibold text-luxury-gold flex items-center gap-2">
              <Filter className="w-5 h-5" />
              篩選器
            </h3>
            <button onClick={fetchRegistrations} className="inline-flex items-center gap-2 px-3 py-1.5 text-sm text-luxury-platinum/80 hover:text-luxury-gold hover:bg-white/10 rounded-md transition-colors">
              <RefreshCw className="w-4 h-4" />
              刷新
            </button>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="relative">
              <label className="block text-sm font-medium text-luxury-platinum mb-2">關鍵字搜尋</label>
              <Search className="absolute left-3 top-10 w-4 h-4 text-luxury-gold/60" />
              <input
                type="text"
                placeholder="搜尋活動名稱..."
                value={filters.search}
                onChange={(e) => handleFilterChange('search', e.target.value)}
                className="w-full pl-10 pr-4 py-2 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum placeholder-luxury-platinum/50"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-luxury-platinum mb-2">報名狀態</label>
              <select value={filters.status} onChange={(e) => handleFilterChange('status', e.target.value)} className="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum">
                <option value="">所有狀態</option>
                <option value="pending">審核中</option>
                <option value="approved">已核准</option>
                <option value="rejected">已婉拒</option>
                <option value="cancelled">已取消</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-luxury-platinum mb-2">付款狀態</label>
              <select value={filters.paymentStatus} onChange={(e) => handleFilterChange('paymentStatus', e.target.value)} className="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum">
                <option value="">所有狀態</option>
                <option value="pending">待付款</option>
                <option value="paid">已付款</option>
                <option value="refunded">已退款</option>
              </select>
            </div>
            <div className="flex items-end">
              <button onClick={clearFilters} className="w-full px-4 py-2 text-luxury-platinum bg-white/10 border border-white/20 rounded-lg hover:bg-white/20 transition-colors">
                清除篩選
              </button>
            </div>
          </div>
        </div>

        <div className="luxury-glass rounded-xl shadow-2xl overflow-hidden">
          <div className="px-6 py-5 border-b border-white/20 bg-white/5">
            <h3 className="text-xl font-luxury font-semibold text-luxury-gold">
              報名總覽 <span className="text-luxury-platinum/80">({totalRegistrations} 筆記錄)</span>
            </h3>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-20 gap-3">
              <Loader2 className="w-8 h-8 text-luxury-gold animate-spin" />
              <span className="text-lg text-luxury-platinum">讀取報名記錄中...</span>
            </div>
          ) : registrations.length === 0 ? (
            <div className="text-center py-20">
              <Ticket className="w-20 h-20 text-luxury-gold/50 mx-auto mb-6" />
              <h3 className="text-2xl font-luxury font-semibold text-luxury-gold mb-3">尚無報名記錄</h3>
              <p className="text-luxury-platinum/80 mb-6">您尚未報名任何活動，立即探索我們的精選活動吧！</p>
              <button onClick={() => navigate('/events')} className="luxury-button inline-flex items-center gap-2">
                <ExternalLink className="w-5 h-5" />
                探索活動
              </button>
            </div>
          ) : (
            <div className="divide-y divide-white/20">
              {registrations.map((reg) => (
                <div key={reg.id} className="p-6 hover:bg-white/5 transition-colors duration-300">
                  <div className="flex flex-col md:flex-row items-start justify-between gap-4">
                    <div className="flex-1">
                      <h4 className="text-xl font-luxury font-bold text-luxury-gold mb-3">{reg.eventName}</h4>
                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 mb-4 text-luxury-platinum">
                        <div className="flex items-center gap-2"><Calendar className="w-4 h-4 text-luxury-gold" /><span>{formatDateTime(reg.eventDateTime)}</span></div>
                        <div className="flex items-center gap-2"><MapPin className="w-4 h-4 text-luxury-gold" /><span>{reg.venueName}</span></div>
                        <div className="flex items-center gap-2"><Clock className="w-4 h-4 text-luxury-gold" /><span>報名於 {formatDateTime(reg.createdAt)}</span></div>
                      </div>
                      <div className="flex items-center gap-3 mb-4">
                        {getStatusBadge(reg.status)}
                        {getPaymentBadge(reg.paymentStatus)}
                      </div>
                      {reg.specialRequests && (
                        <div className="luxury-glass rounded-lg p-3">
                          <p className="text-sm font-medium text-luxury-platinum mb-1">特別要求:</p>
                          <p className="text-sm text-luxury-platinum/80 whitespace-pre-wrap">{reg.specialRequests}</p>
                        </div>
                      )}
                    </div>
                    <div className="flex items-center gap-2 self-start md:self-center mt-4 md:mt-0">
                      <button onClick={() => navigate(`/events/${reg.eventId}`)} className="p-2 text-luxury-platinum/60 hover:text-luxury-gold hover:bg-white/10 rounded-full transition-colors" title="查看活動詳情"><Eye className="w-5 h-5" /></button>
                      {canEdit(reg) && <button onClick={() => openEditModal(reg)} className="p-2 text-blue-400 hover:text-blue-300 hover:bg-blue-900/50 rounded-full transition-colors" title="編輯報名"><Edit className="w-5 h-5" /></button>}
                      {canCancel(reg) && <button onClick={() => handleCancelRegistration(reg.id)} disabled={actionLoading === `cancel-${reg.id}`} className="p-2 text-red-400 hover:text-red-300 hover:bg-red-900/50 rounded-full transition-colors disabled:opacity-50" title="取消報名"><X className="w-5 h-5" /></button>}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {totalPages > 1 && (
            <div className="px-6 py-4 border-t border-white/20 bg-white/5 flex items-center justify-between">
              <div className="text-sm text-luxury-platinum/80">
                顯示第 {((currentPage - 1) * pageSize) + 1} 至 {Math.min(currentPage * pageSize, totalRegistrations)} 項，共 {totalRegistrations} 項結果
              </div>
              <div className="flex items-center gap-2">
                <button onClick={() => setCurrentPage(p => Math.max(p - 1, 1))} disabled={currentPage === 1} className="inline-flex items-center px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-sm font-medium text-luxury-platinum hover:bg-white/20 disabled:opacity-50 disabled:cursor-not-allowed">
                  <ChevronLeft className="w-4 h-4" />
                  上一頁
                </button>
                <span className="text-sm text-luxury-platinum">
                  第 {currentPage} / {totalPages} 頁
                </span>
                <button onClick={() => setCurrentPage(p => Math.min(p + 1, totalPages))} disabled={currentPage === totalPages} className="inline-flex items-center px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-sm font-medium text-luxury-platinum hover:bg-white/20 disabled:opacity-50 disabled:cursor-not-allowed">
                  下一頁
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {showEditModal && selectedRegistration && (
        <div className="fixed inset-0 bg-luxury-midnight-black/90 backdrop-blur-luxury flex items-center justify-center z-50">
          <div className="luxury-glass rounded-xl p-7 w-full max-w-md shadow-2xl">
            <div className="flex items-center justify-between mb-5">
              <h3 className="text-2xl font-luxury font-bold text-luxury-gold">編輯報名資訊</h3>
              <button onClick={() => setShowEditModal(false)} className="text-luxury-platinum/60 hover:text-luxury-gold transition-colors"><X className="w-7 h-7" /></button>
            </div>
            <div className="mb-6">
              <p className="text-luxury-platinum mb-4">
                活動: <strong className="text-luxury-gold">{selectedRegistration.eventName}</strong>
              </p>
              <label className="block text-sm font-medium text-luxury-platinum mb-2">特別要求</label>
              <textarea
                value={editRequests}
                onChange={(e) => setEditRequests(e.target.value)}
                rows={5}
                className="w-full px-4 py-3 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum placeholder-luxury-platinum/50"
                placeholder="請輸入您的飲食限制、過敏資訊或任何特殊需求..."
              />
            </div>
            <div className="flex justify-end gap-4">
              <button onClick={() => setShowEditModal(false)} className="px-5 py-2.5 text-luxury-platinum bg-white/10 border border-white/20 rounded-lg hover:bg-white/20 transition-colors">
                取消
              </button>
              <button onClick={handleEditRegistration} disabled={actionLoading === 'edit'} className="luxury-button px-5 py-2.5 flex items-center gap-2 disabled:opacity-50">
                {actionLoading === 'edit' ? <Loader2 className="w-5 h-5 animate-spin" /> : '儲存變更'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default MyRegistrations;
