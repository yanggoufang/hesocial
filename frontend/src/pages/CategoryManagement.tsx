import React, { useState, useEffect } from 'react'
import { useAuth } from '../hooks/useAuth'
import { 
  Tag, 
  Search, 
  Plus, 
  Edit, 
  Trash2, 
  Eye, 
  EyeOff,
  RefreshCw,
  ArrowUp,
  ArrowDown,
  Save,
  X,
  Palette,
  Users,
  Clock,
  Settings
} from 'lucide-react'

interface EventCategory {
  id: string
  name: string
  slug: string
  description: string
  icon: string
  color: string
  target_membership_tiers: string[]
  typical_duration_hours: number
  typical_capacity: { min: number; max: number }
  is_active: boolean
  sort_order: number
  created_at: string
  updated_at: string
}

interface CreateCategoryData {
  name: string
  description: string
  icon: string
  color: string
  targetMembershipTiers: string[]
  typicalDurationHours: number
  typicalCapacity: { min: number; max: number }
  sortOrder: number
}

const CategoryManagement: React.FC = () => {
  const { user } = useAuth()
  const [categories, setCategories] = useState<EventCategory[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [searchTerm, setSearchTerm] = useState('')
  
  // Modals
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showEditModal, setShowEditModal] = useState(false)
  const [showDeleteModal, setShowDeleteModal] = useState(false)
  const [selectedCategory, setSelectedCategory] = useState<EventCategory | null>(null)
  
  // Form data
  const [createData, setCreateData] = useState<CreateCategoryData>({
    name: '',
    description: '',
    icon: 'Tag',
    color: '#8B5CF6',
    targetMembershipTiers: [],
    typicalDurationHours: 3,
    typicalCapacity: { min: 8, max: 20 },
    sortOrder: 0
  })
  
  // Loading states
  const [actionLoading, setActionLoading] = useState<string | null>(null)

  useEffect(() => {
    fetchCategories()
  }, [])

  const fetchCategories = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await fetch('/api/categories', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        }
      })
      
      const data = await response.json()
      
      if (data.success) {
        setCategories(data.data || [])
      } else {
        setError(data.error || 'Failed to fetch categories')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const handleCreateCategory = async () => {
    setActionLoading('create')
    
    try {
      const response = await fetch('/api/categories', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        },
        body: JSON.stringify(createData)
      })
      
      const data = await response.json()
      
      if (data.success) {
        setShowCreateModal(false)
        setCreateData({
          name: '',
          description: '',
          icon: 'Tag',
          color: '#8B5CF6',
          targetMembershipTiers: [],
          typicalDurationHours: 3,
          typicalCapacity: { min: 8, max: 20 },
          sortOrder: 0
        })
        fetchCategories()
      } else {
        setError(data.error || 'Failed to create category')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleUpdateCategory = async (updates: Partial<EventCategory>) => {
    if (!selectedCategory) return
    
    setActionLoading('update')
    
    try {
      const response = await fetch(`/api/categories/${selectedCategory.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        },
        body: JSON.stringify(updates)
      })
      
      const data = await response.json()
      
      if (data.success) {
        setShowEditModal(false)
        fetchCategories()
      } else {
        setError(data.error || 'Failed to update category')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleDeleteCategory = async () => {
    if (!selectedCategory) return
    
    setActionLoading('delete')
    
    try {
      const response = await fetch(`/api/categories/${selectedCategory.id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        }
      })
      
      const data = await response.json()
      
      if (data.success) {
        setShowDeleteModal(false)
        fetchCategories()
      } else {
        setError(data.error || 'Failed to delete category')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleToggleActive = async (category: EventCategory) => {
    setActionLoading(`toggle-${category.id}`)
    
    try {
      const response = await fetch(`/api/categories/${category.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        },
        body: JSON.stringify({ is_active: !category.is_active })
      })
      
      const data = await response.json()
      
      if (data.success) {
        fetchCategories()
      } else {
        setError(data.error || 'Failed to update category status')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleSortOrderChange = async (category: EventCategory, direction: 'up' | 'down') => {
    const newOrder = direction === 'up' ? category.sort_order - 1 : category.sort_order + 1
    setActionLoading(`sort-${category.id}`)
    
    try {
      const response = await fetch(`/api/categories/${category.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        },
        body: JSON.stringify({ sort_order: newOrder })
      })
      
      const data = await response.json()
      
      if (data.success) {
        fetchCategories()
      } else {
        setError(data.error || 'Failed to update sort order')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const filteredCategories = categories.filter(category =>
    category.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    category.description.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const membershipTierOptions = ['Platinum', 'Diamond', 'Black Card']
  const iconOptions = ['Tag', 'Users', 'Crown', 'Star', 'Award', 'Calendar', 'MapPin', 'Camera']
  const colorOptions = [
    '#8B5CF6', '#EF4444', '#F59E0B', '#10B981', '#3B82F6', 
    '#6366F1', '#8B5A2B', '#EC4899', '#6B7280', '#1F2937'
  ]

  if (!user || (user.role !== 'admin' && user.role !== 'super_admin')) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <Tag className="w-16 h-16 text-gray-400 mx-auto mb-4" />
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
                <Tag className="w-8 h-8 text-purple-600" />
                Event Category Management
              </h1>
              <p className="text-gray-600 mt-2">Manage luxury event categories and their settings</p>
            </div>
            <div className="flex items-center gap-3">
              <button
                onClick={fetchCategories}
                className="inline-flex items-center gap-2 px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
              >
                <RefreshCw className="w-4 h-4" />
                Refresh
              </button>
              <button
                onClick={() => setShowCreateModal(true)}
                className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
              >
                <Plus className="w-4 h-4" />
                Create Category
              </button>
            </div>
          </div>
        </div>

        {/* Error Message */}
        {error && (
          <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4">
            <div className="flex items-center">
              <X className="w-5 h-5 text-red-600 mr-2" />
              <p className="text-red-800">{error}</p>
              <button
                onClick={() => setError(null)}
                className="ml-auto text-red-600 hover:text-red-800"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Search */}
        <div className="bg-white rounded-lg shadow-sm border mb-6 p-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              placeholder="Search categories..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
            />
          </div>
        </div>

        {/* Categories Grid */}
        <div className="bg-white rounded-lg shadow-sm border overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">
              Event Categories ({filteredCategories.length} total)
            </h3>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-12">
              <RefreshCw className="w-6 h-6 text-gray-400 animate-spin" />
              <span className="ml-2 text-gray-600">Loading categories...</span>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-6">
              {filteredCategories.map((category) => (
                <div key={category.id} className="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-3">
                      <div 
                        className="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-medium"
                        style={{ backgroundColor: category.color }}
                      >
                        <Tag className="w-4 h-4" />
                      </div>
                      <div>
                        <h4 className="text-lg font-medium text-gray-900">{category.name}</h4>
                        <p className="text-sm text-gray-500">#{category.sort_order}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => handleSortOrderChange(category, 'up')}
                        disabled={actionLoading === `sort-${category.id}`}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-50"
                        title="Move Up"
                      >
                        <ArrowUp className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => handleSortOrderChange(category, 'down')}
                        disabled={actionLoading === `sort-${category.id}`}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-50"
                        title="Move Down"
                      >
                        <ArrowDown className="w-4 h-4" />
                      </button>
                    </div>
                  </div>

                  <p className="text-sm text-gray-700 mb-3 line-clamp-2">{category.description}</p>

                  <div className="space-y-2 mb-4">
                    <div className="flex items-center gap-2 text-xs text-gray-600">
                      <Users className="w-3 h-3" />
                      <span>Target: {category.target_membership_tiers?.join(', ') || 'All tiers'}</span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-gray-600">
                      <Clock className="w-3 h-3" />
                      <span>{category.typical_duration_hours}h duration</span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-gray-600">
                      <Settings className="w-3 h-3" />
                      <span>
                        {category.typical_capacity?.min || 0}-{category.typical_capacity?.max || 0} capacity
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      {category.is_active ? (
                        <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                          <Eye className="w-3 h-3" />
                          Active
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                          <EyeOff className="w-3 h-3" />
                          Inactive
                        </span>
                      )}
                    </div>
                    
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => handleToggleActive(category)}
                        disabled={actionLoading === `toggle-${category.id}`}
                        className="p-1 text-gray-400 hover:text-blue-600"
                        title={category.is_active ? 'Deactivate' : 'Activate'}
                      >
                        {category.is_active ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                      <button
                        onClick={() => {
                          setSelectedCategory(category)
                          setShowEditModal(true)
                        }}
                        className="p-1 text-gray-400 hover:text-blue-600"
                        title="Edit"
                      >
                        <Edit className="w-4 h-4" />
                      </button>
                      {user.role === 'super_admin' && (
                        <button
                          onClick={() => {
                            setSelectedCategory(category)
                            setShowDeleteModal(true)
                          }}
                          className="p-1 text-gray-400 hover:text-red-600"
                          title="Delete"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Create Category Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-gray-900">Create Event Category</h3>
              <button
                onClick={() => setShowCreateModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="w-6 h-6" />
              </button>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-700 mb-1">Category Name</label>
                <input
                  type="text"
                  value={createData.name}
                  onChange={(e) => setCreateData(prev => ({ ...prev, name: e.target.value }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  placeholder="e.g., Private Dining Experiences"
                />
              </div>
              
              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                <textarea
                  value={createData.description}
                  onChange={(e) => setCreateData(prev => ({ ...prev, description: e.target.value }))}
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  placeholder="Describe this event category..."
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Icon</label>
                <select
                  value={createData.icon}
                  onChange={(e) => setCreateData(prev => ({ ...prev, icon: e.target.value }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                >
                  {iconOptions.map(icon => (
                    <option key={icon} value={icon}>{icon}</option>
                  ))}
                </select>
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Color</label>
                <div className="flex gap-2 mt-1">
                  {colorOptions.map(color => (
                    <button
                      key={color}
                      onClick={() => setCreateData(prev => ({ ...prev, color }))}
                      className={`w-8 h-8 rounded-full border-2 ${createData.color === color ? 'border-gray-900' : 'border-gray-300'}`}
                      style={{ backgroundColor: color }}
                    />
                  ))}
                </div>
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Duration (hours)</label>
                <input
                  type="number"
                  value={createData.typicalDurationHours}
                  onChange={(e) => setCreateData(prev => ({ ...prev, typicalDurationHours: parseInt(e.target.value) }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  min="1"
                  max="24"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Sort Order</label>
                <input
                  type="number"
                  value={createData.sortOrder}
                  onChange={(e) => setCreateData(prev => ({ ...prev, sortOrder: parseInt(e.target.value) }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  min="0"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Min Capacity</label>
                <input
                  type="number"
                  value={createData.typicalCapacity.min}
                  onChange={(e) => setCreateData(prev => ({ 
                    ...prev, 
                    typicalCapacity: { ...prev.typicalCapacity, min: parseInt(e.target.value) }
                  }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  min="1"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Max Capacity</label>
                <input
                  type="number"
                  value={createData.typicalCapacity.max}
                  onChange={(e) => setCreateData(prev => ({ 
                    ...prev, 
                    typicalCapacity: { ...prev.typicalCapacity, max: parseInt(e.target.value) }
                  }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  min="1"
                />
              </div>
              
              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-700 mb-1">Target Membership Tiers</label>
                <div className="flex gap-3">
                  {membershipTierOptions.map(tier => (
                    <label key={tier} className="flex items-center">
                      <input
                        type="checkbox"
                        checked={createData.targetMembershipTiers.includes(tier)}
                        onChange={(e) => {
                          const tiers = e.target.checked
                            ? [...createData.targetMembershipTiers, tier]
                            : createData.targetMembershipTiers.filter(t => t !== tier)
                          setCreateData(prev => ({ ...prev, targetMembershipTiers: tiers }))
                        }}
                        className="mr-2 rounded border-gray-300 text-purple-600 focus:ring-purple-500"
                      />
                      <span className="text-sm text-gray-700">{tier}</span>
                    </label>
                  ))}
                </div>
              </div>
            </div>
            
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateCategory}
                disabled={actionLoading === 'create' || !createData.name.trim()}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2"
              >
                {actionLoading === 'create' && <RefreshCw className="w-4 h-4 animate-spin" />}
                Create Category
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit Category Modal */}
      {showEditModal && selectedCategory && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-gray-900">Edit Event Category</h3>
              <button
                onClick={() => setShowEditModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="w-6 h-6" />
              </button>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-700 mb-1">Category Name</label>
                <input
                  type="text"
                  defaultValue={selectedCategory.name}
                  id="edit-name"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              
              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                <textarea
                  defaultValue={selectedCategory.description}
                  id="edit-description"
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Duration (hours)</label>
                <input
                  type="number"
                  defaultValue={selectedCategory.typical_duration_hours}
                  id="edit-duration"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  min="1"
                  max="24"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Sort Order</label>
                <input
                  type="number"
                  defaultValue={selectedCategory.sort_order}
                  id="edit-sort-order"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  min="0"
                />
              </div>
            </div>
            
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowEditModal(false)}
                className="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  const name = (document.getElementById('edit-name') as HTMLInputElement).value
                  const description = (document.getElementById('edit-description') as HTMLTextAreaElement).value
                  const duration = parseInt((document.getElementById('edit-duration') as HTMLInputElement).value)
                  const sortOrder = parseInt((document.getElementById('edit-sort-order') as HTMLInputElement).value)
                  
                  handleUpdateCategory({
                    name,
                    description,
                    typical_duration_hours: duration,
                    sort_order: sortOrder
                  })
                }}
                disabled={actionLoading === 'update'}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2"
              >
                {actionLoading === 'update' && <RefreshCw className="w-4 h-4 animate-spin" />}
                Save Changes
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {showDeleteModal && selectedCategory && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-bold text-gray-900">Confirm Delete</h3>
              <button
                onClick={() => setShowDeleteModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            
            <p className="text-gray-700 mb-6">
              Are you sure you want to delete the category <strong>{selectedCategory.name}</strong>? This action cannot be undone.
            </p>
            
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setShowDeleteModal(false)}
                className="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleDeleteCategory}
                disabled={actionLoading === 'delete'}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 flex items-center gap-2"
              >
                {actionLoading === 'delete' && <RefreshCw className="w-4 h-4 animate-spin" />}
                Delete Category
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default CategoryManagement