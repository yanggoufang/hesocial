import React, { useState, useEffect } from 'react'
import { useAuth } from '../hooks/useAuth'
import { adminService, User, UserStats, AdminResponse } from '../services/adminService'
import { 
  Users, 
  Search, 
  Filter, 
  Edit, 
  Trash2, 
  CheckCircle, 
  XCircle, 
  Shield, 
  UserCheck, 
  Eye,
  ChevronLeft,
  ChevronRight,
  Download,
  RefreshCw,
  Settings,
  AlertTriangle,
  Crown,
  Star
} from 'lucide-react'

interface UserFilters {
  search: string
  role: string
  membershipTier: string
  verificationStatus: string
}

interface EditUserData {
  firstName: string
  lastName: string
  age: number
  profession: string
  annualIncome: number
  netWorth: number
  membershipTier: 'Platinum' | 'Diamond' | 'Black Card'
  privacyLevel: 1 | 2 | 3 | 4 | 5
  bio: string
  interests: string[]
}

const UserManagement: React.FC = () => {
  const { user } = useAuth()
  const [users, setUsers] = useState<User[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [stats, setStats] = useState<UserStats | null>(null)
  
  // Pagination
  const [currentPage, setCurrentPage] = useState(1)
  const [totalPages, setTotalPages] = useState(1)
  const [totalUsers, setTotalUsers] = useState(0)
  const pageSize = 20

  // Filters
  const [filters, setFilters] = useState<UserFilters>({
    search: '',
    role: '',
    membershipTier: '',
    verificationStatus: ''
  })
  const [showFilters, setShowFilters] = useState(false)

  // Modals
  const [selectedUser, setSelectedUser] = useState<User | null>(null)
  const [showUserDetail, setShowUserDetail] = useState(false)
  const [showEditUser, setShowEditUser] = useState(false)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const [editUserData, setEditUserData] = useState<EditUserData | null>(null)

  // Loading states
  const [actionLoading, setActionLoading] = useState<string | null>(null)

  useEffect(() => {
    fetchUsers()
    fetchUserStats()
  }, [currentPage, filters])

  const fetchUsers = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await adminService.getUsers({
        page: currentPage,
        limit: pageSize,
        search: filters.search || undefined,
        role: filters.role || undefined,
        membershipTier: filters.membershipTier || undefined,
        verificationStatus: filters.verificationStatus || undefined
      })

      if (response.success && response.data) {
        setUsers(response.data)
        if (response.pagination) {
          setTotalPages(response.pagination.totalPages)
          setTotalUsers(response.pagination.total)
        }
      } else {
        setError(response.error || 'Failed to fetch users')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const fetchUserStats = async () => {
    try {
      const response = await adminService.getUserStats()
      if (response.success && response.data) {
        setStats(response.data)
      }
    } catch (err) {
      console.error('Failed to fetch user stats:', err)
    }
  }

  const handleFilterChange = (key: keyof UserFilters, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value }))
    setCurrentPage(1) // Reset to first page when filtering
  }

  const clearFilters = () => {
    setFilters({
      search: '',
      role: '',
      membershipTier: '',
      verificationStatus: ''
    })
    setCurrentPage(1)
  }

  const openUserDetail = (user: User) => {
    setSelectedUser(user)
    setShowUserDetail(true)
  }

  const openEditUser = (user: User) => {
    setSelectedUser(user)
    setEditUserData({
      firstName: user.firstName,
      lastName: user.lastName,
      age: user.age,
      profession: user.profession,
      annualIncome: user.annualIncome,
      netWorth: user.netWorth,
      membershipTier: user.membershipTier,
      privacyLevel: user.privacyLevel,
      bio: user.bio || '',
      interests: user.interests
    })
    setShowEditUser(true)
  }

  const handleEditUser = async () => {
    if (!selectedUser || !editUserData) return
    
    setActionLoading('edit')
    try {
      const response = await adminService.updateUser(selectedUser.id, editUserData)
      if (response.success) {
        setShowEditUser(false)
        fetchUsers() // Refresh the list
      } else {
        setError(response.error || 'Failed to update user')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleVerifyUser = async (userId: string, status: 'approved' | 'rejected') => {
    setActionLoading(`verify-${userId}`)
    try {
      const response = await adminService.verifyUser(userId, status)
      if (response.success) {
        fetchUsers() // Refresh the list
      } else {
        setError(response.error || 'Failed to verify user')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleUpdateRole = async (userId: string, role: 'user' | 'admin' | 'super_admin') => {
    setActionLoading(`role-${userId}`)
    try {
      const response = await adminService.updateUserRole(userId, role)
      if (response.success) {
        fetchUsers() // Refresh the list
      } else {
        setError(response.error || 'Failed to update user role')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleDeleteUser = async () => {
    if (!selectedUser) return
    
    setActionLoading('delete')
    try {
      const response = await adminService.deleteUser(selectedUser.id)
      if (response.success) {
        setShowDeleteConfirm(false)
        fetchUsers() // Refresh the list
      } else {
        setError(response.error || 'Failed to delete user')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
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

  const getVerificationBadge = (status: string, isVerified: boolean) => {
    if (status === 'approved' && isVerified) {
      return <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800 border border-green-300">
        <CheckCircle className="w-3 h-3" />
        Verified
      </span>
    }
    if (status === 'rejected') {
      return <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800 border border-red-300">
        <XCircle className="w-3 h-3" />
        Rejected
      </span>
    }
    return <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 border border-yellow-300">
      <AlertTriangle className="w-3 h-3" />
      Pending
    </span>
  }

  const getRoleBadge = (role: string) => {
    const styles = {
      'user': 'bg-gray-100 text-gray-800',
      'admin': 'bg-purple-100 text-purple-800',
      'super_admin': 'bg-red-100 text-red-800'
    }

    return (
      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${styles[role as keyof typeof styles]}`}>
        {role === 'super_admin' ? 'Super Admin' : role.charAt(0).toUpperCase() + role.slice(1)}
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
          <h1 className="text-2xl font-bold text-gray-900 mb-2">Access Denied</h1>
          <p className="text-gray-600">You don't have permission to access this page.</p>
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
                <Users className="w-8 h-8 text-purple-600" />
                User Management
              </h1>
              <p className="text-gray-600 mt-2">Manage platform members and their permissions</p>
            </div>
            <button
              onClick={fetchUsers}
              className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              Refresh
            </button>
          </div>

          {/* Stats Cards */}
          {stats && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mt-6">
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">Total Users</p>
                    <p className="text-3xl font-bold text-gray-900">{stats.totalUsers}</p>
                  </div>
                  <Users className="w-8 h-8 text-blue-600" />
                </div>
              </div>
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">Recent Registrations</p>
                    <p className="text-3xl font-bold text-gray-900">{stats.recentRegistrations}</p>
                  </div>
                  <UserCheck className="w-8 h-8 text-green-600" />
                </div>
              </div>
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">Pending Verification</p>
                    <p className="text-3xl font-bold text-gray-900">
                      {stats.usersByVerificationStatus.find(s => s.verification_status === 'pending')?.count || 0}
                    </p>
                  </div>
                  <AlertTriangle className="w-8 h-8 text-yellow-600" />
                </div>
              </div>
              <div className="bg-white rounded-lg p-6 shadow-sm border">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600">Admins</p>
                    <p className="text-3xl font-bold text-gray-900">
                      {(stats.usersByRole.find(r => r.role === 'admin')?.count || 0) + 
                       (stats.usersByRole.find(r => r.role === 'super_admin')?.count || 0)}
                    </p>
                  </div>
                  <Shield className="w-8 h-8 text-red-600" />
                </div>
              </div>
            </div>
          )}
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

        {/* Filters */}
        <div className="bg-white rounded-lg shadow-sm border mb-6">
          <div className="p-4 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-medium text-gray-900">Filters & Search</h3>
              <button
                onClick={() => setShowFilters(!showFilters)}
                className="inline-flex items-center gap-2 px-3 py-1 text-sm text-gray-600 hover:text-gray-900"
              >
                <Filter className="w-4 h-4" />
                {showFilters ? 'Hide Filters' : 'Show Filters'}
              </button>
            </div>
          </div>
          
          <div className="p-4">
            {/* Search */}
            <div className="relative mb-4">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                placeholder="Search by name, email, profession..."
                value={filters.search}
                onChange={(e) => handleFilterChange('search', e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              />
            </div>

            {/* Advanced Filters */}
            {showFilters && (
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Role</label>
                  <select
                    value={filters.role}
                    onChange={(e) => handleFilterChange('role', e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  >
                    <option value="">All Roles</option>
                    <option value="user">User</option>
                    <option value="admin">Admin</option>
                    <option value="super_admin">Super Admin</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Membership Tier</label>
                  <select
                    value={filters.membershipTier}
                    onChange={(e) => handleFilterChange('membershipTier', e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  >
                    <option value="">All Tiers</option>
                    <option value="Platinum">Platinum</option>
                    <option value="Diamond">Diamond</option>
                    <option value="Black Card">Black Card</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Verification Status</label>
                  <select
                    value={filters.verificationStatus}
                    onChange={(e) => handleFilterChange('verificationStatus', e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  >
                    <option value="">All Statuses</option>
                    <option value="pending">Pending</option>
                    <option value="approved">Approved</option>
                    <option value="rejected">Rejected</option>
                  </select>
                </div>
              </div>
            )}

            {/* Clear Filters */}
            {(filters.search || filters.role || filters.membershipTier || filters.verificationStatus) && (
              <div className="mt-4 flex justify-end">
                <button
                  onClick={clearFilters}
                  className="px-4 py-2 text-sm text-gray-600 hover:text-gray-900 border border-gray-300 rounded-lg hover:bg-gray-50"
                >
                  Clear All Filters
                </button>
              </div>
            )}
          </div>
        </div>

        {/* Users Table */}
        <div className="bg-white rounded-lg shadow-sm border overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-medium text-gray-900">
                Users ({totalUsers} total)
              </h3>
              <button className="inline-flex items-center gap-2 px-3 py-1 text-sm text-gray-600 hover:text-gray-900 border border-gray-300 rounded-lg hover:bg-gray-50">
                <Download className="w-4 h-4" />
                Export
              </button>
            </div>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-12">
              <RefreshCw className="w-6 h-6 text-gray-400 animate-spin" />
              <span className="ml-2 text-gray-600">Loading users...</span>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">User</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Membership</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Role</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Financial</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Joined</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {users.map((user) => (
                    <tr key={user.id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex items-center">
                          <div className="flex-shrink-0 h-10 w-10">
                            {user.profilePicture ? (
                              <img className="h-10 w-10 rounded-full" src={user.profilePicture} alt="" />
                            ) : (
                              <div className="h-10 w-10 rounded-full bg-gray-300 flex items-center justify-center">
                                <span className="text-sm font-medium text-gray-700">
                                  {user.firstName.charAt(0)}{user.lastName.charAt(0)}
                                </span>
                              </div>
                            )}
                          </div>
                          <div className="ml-4">
                            <div className="text-sm font-medium text-gray-900">{user.firstName} {user.lastName}</div>
                            <div className="text-sm text-gray-500">{user.email}</div>
                            <div className="text-xs text-gray-400">{user.profession}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        {getMembershipTierBadge(user.membershipTier)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        {getVerificationBadge(user.verificationStatus, user.isVerified)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        {getRoleBadge(user.role)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <div>Income: {formatCurrency(user.annualIncome)}</div>
                        <div className="text-gray-500">Net Worth: {formatCurrency(user.netWorth)}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {new Date(user.createdAt).toLocaleDateString()}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                        <div className="flex items-center justify-end gap-2">
                          <button
                            onClick={() => openUserDetail(user)}
                            className="text-gray-600 hover:text-gray-900"
                            title="View Details"
                          >
                            <Eye className="w-4 h-4" />
                          </button>
                          <button
                            onClick={() => openEditUser(user)}
                            className="text-blue-600 hover:text-blue-900"
                            title="Edit User"
                          >
                            <Edit className="w-4 h-4" />
                          </button>
                          {user.verificationStatus === 'pending' && (
                            <>
                              <button
                                onClick={() => handleVerifyUser(user.id, 'approved')}
                                disabled={actionLoading === `verify-${user.id}`}
                                className="text-green-600 hover:text-green-900 disabled:opacity-50"
                                title="Approve"
                              >
                                <CheckCircle className="w-4 h-4" />
                              </button>
                              <button
                                onClick={() => handleVerifyUser(user.id, 'rejected')}
                                disabled={actionLoading === `verify-${user.id}`}
                                className="text-red-600 hover:text-red-900 disabled:opacity-50"
                                title="Reject"
                              >
                                <XCircle className="w-4 h-4" />
                              </button>
                            </>
                          )}
                          {user?.role === 'super_admin' && (
                            <button
                              onClick={() => {
                                setSelectedUser(user)
                                setShowDeleteConfirm(true)
                              }}
                              className="text-red-600 hover:text-red-900"
                              title="Delete User"
                            >
                              <Trash2 className="w-4 h-4" />
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="px-6 py-4 border-t border-gray-200 flex items-center justify-between">
              <div className="text-sm text-gray-700">
                Showing {((currentPage - 1) * pageSize) + 1} to {Math.min(currentPage * pageSize, totalUsers)} of {totalUsers} results
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
                  disabled={currentPage === 1}
                  className="inline-flex items-center px-3 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <ChevronLeft className="w-4 h-4" />
                  Previous
                </button>
                <span className="text-sm text-gray-700">
                  Page {currentPage} of {totalPages}
                </span>
                <button
                  onClick={() => setCurrentPage(prev => Math.min(prev + 1, totalPages))}
                  disabled={currentPage === totalPages}
                  className="inline-flex items-center px-3 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Next
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* User Detail Modal */}
      {showUserDetail && selectedUser && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-gray-900">User Details</h3>
              <button
                onClick={() => setShowUserDetail(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <XCircle className="w-6 h-6" />
              </button>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h4 className="font-medium text-gray-900 mb-3">Personal Information</h4>
                <div className="space-y-2 text-sm">
                  <p><span className="font-medium">Name:</span> {selectedUser.firstName} {selectedUser.lastName}</p>
                  <p><span className="font-medium">Email:</span> {selectedUser.email}</p>
                  <p><span className="font-medium">Age:</span> {selectedUser.age}</p>
                  <p><span className="font-medium">Profession:</span> {selectedUser.profession}</p>
                  <p><span className="font-medium">Privacy Level:</span> {selectedUser.privacyLevel}/5</p>
                </div>
              </div>
              
              <div>
                <h4 className="font-medium text-gray-900 mb-3">Account Information</h4>
                <div className="space-y-2 text-sm">
                  <p><span className="font-medium">Membership:</span> {getMembershipTierBadge(selectedUser.membershipTier)}</p>
                  <p><span className="font-medium">Role:</span> {getRoleBadge(selectedUser.role)}</p>
                  <p><span className="font-medium">Status:</span> {getVerificationBadge(selectedUser.verificationStatus, selectedUser.isVerified)}</p>
                  <p><span className="font-medium">Joined:</span> {new Date(selectedUser.createdAt).toLocaleDateString()}</p>
                </div>
              </div>
              
              <div>
                <h4 className="font-medium text-gray-900 mb-3">Financial Information</h4>
                <div className="space-y-2 text-sm">
                  <p><span className="font-medium">Annual Income:</span> {formatCurrency(selectedUser.annualIncome)}</p>
                  <p><span className="font-medium">Net Worth:</span> {formatCurrency(selectedUser.netWorth)}</p>
                </div>
              </div>
              
              <div>
                <h4 className="font-medium text-gray-900 mb-3">Interests</h4>
                <div className="flex flex-wrap gap-2">
                  {selectedUser.interests.map((interest, index) => (
                    <span key={index} className="px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs">
                      {interest}
                    </span>
                  ))}
                </div>
              </div>
            </div>
            
            {selectedUser.bio && (
              <div className="mt-6">
                <h4 className="font-medium text-gray-900 mb-3">Bio</h4>
                <p className="text-sm text-gray-700">{selectedUser.bio}</p>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Edit User Modal */}
      {showEditUser && selectedUser && editUserData && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-gray-900">Edit User</h3>
              <button
                onClick={() => setShowEditUser(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <XCircle className="w-6 h-6" />
              </button>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">First Name</label>
                <input
                  type="text"
                  value={editUserData.firstName}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, firstName: e.target.value} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Last Name</label>
                <input
                  type="text"
                  value={editUserData.lastName}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, lastName: e.target.value} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Age</label>
                <input
                  type="number"
                  value={editUserData.age}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, age: parseInt(e.target.value)} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Profession</label>
                <input
                  type="text"
                  value={editUserData.profession}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, profession: e.target.value} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Annual Income (TWD)</label>
                <input
                  type="number"
                  value={editUserData.annualIncome}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, annualIncome: parseInt(e.target.value)} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Net Worth (TWD)</label>
                <input
                  type="number"
                  value={editUserData.netWorth}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, netWorth: parseInt(e.target.value)} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Membership Tier</label>
                <select
                  value={editUserData.membershipTier}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, membershipTier: e.target.value as any} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                >
                  <option value="Platinum">Platinum</option>
                  <option value="Diamond">Diamond</option>
                  <option value="Black Card">Black Card</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Privacy Level</label>
                <select
                  value={editUserData.privacyLevel}
                  onChange={(e) => setEditUserData(prev => prev ? {...prev, privacyLevel: parseInt(e.target.value) as any} : null)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                >
                  <option value={1}>1 - Public</option>
                  <option value={2}>2 - Low</option>
                  <option value={3}>3 - Medium</option>
                  <option value={4}>4 - High</option>
                  <option value={5}>5 - Very High</option>
                </select>
              </div>
            </div>
            
            <div className="mt-4">
              <label className="block text-sm font-medium text-gray-700 mb-1">Bio</label>
              <textarea
                value={editUserData.bio}
                onChange={(e) => setEditUserData(prev => prev ? {...prev, bio: e.target.value} : null)}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              />
            </div>
            
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowEditUser(false)}
                className="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleEditUser}
                disabled={actionLoading === 'edit'}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2"
              >
                {actionLoading === 'edit' && <RefreshCw className="w-4 h-4 animate-spin" />}
                Save Changes
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && selectedUser && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-bold text-gray-900">Confirm Delete</h3>
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <XCircle className="w-5 h-5" />
              </button>
            </div>
            
            <p className="text-gray-700 mb-6">
              Are you sure you want to delete user <strong>{selectedUser.firstName} {selectedUser.lastName}</strong>? This action cannot be undone.
            </p>
            
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleDeleteUser}
                disabled={actionLoading === 'delete'}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
              >
                {actionLoading === 'delete' && <RefreshCw className="w-4 h-4 animate-spin" />}
                Delete User
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default UserManagement