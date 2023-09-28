#include <gqlite.h>

#include <algorithm>
#include <iostream>
#include <span>
#include <string>
#include <unordered_map>
#include <vector>

#include "gqlite_p.h"
#include "errors.h"

namespace gqlite
{
  template<typename _T_>
  void swap_span_content(std::span<_T_> _a, std::span<_T_> _b)
  {
    swap_ranges(_a.begin(), _a.end(), _b.begin());
  }
  /**
   * @internal
   * 
   * This class allow to store data as a table, during the execution of the query.
   */
  template<typename _T_>
  class table
  {
  public:
    class const_row_view : public std::span<const _T_>
    {
      using std::span<const _T_>::span;
    public:
      operator std::vector<_T_>() const
      {
        return std::vector<_T_>(this->begin(), this->end());
      }
      template<template<typename T> class C>
      bool operator==(const C<_T_>& _v) const
      {
        return this->size() == _v.size() and std::equal(this->begin(), this->end(), _v.begin(), _v.end());
      }
    };
    class row_view : public std::span<_T_>
    {
      using std::span<_T_>::span;
    public:
      operator std::vector<_T_>() const
      {
        return std::vector<_T_>(this->begin(), this->end());
      }
      row_view& operator=(const std::vector<_T_>& _rhs)
      {
        errors::check_condition(_rhs.size() == this->size(), "Cannot assign a vector of size {} to a row view of size {}", _rhs.size(), this->size());
        std::copy(_rhs.begin(), _rhs.end(), this->begin());
        return *this;
      }
      template<template<typename T> class C>
      bool operator==(const C<_T_>& _v) const
      {
        return this->size() == _v.size() and std::equal(this->begin(), this->end(), _v.begin(), _v.end());
      }
    };
    class iterator
    {
      friend class table;
      iterator(std::size_t _row, table* _table) : m_row(_row), m_table_ptr(_table) {}
    public:
      using iterator_category = std::random_access_iterator_tag;
      using value_type = std::vector<_T_>;
      using difference_type = std::ptrdiff_t;
      using pointer = row_view;
      using reference = row_view;
      iterator& operator++()
      {
        ++m_row;
        return *this;
      }
      iterator& operator--()
      {
        --m_row;
        return *this;
      }
      row_view operator*()
      {
        return m_table_ptr->get_row(m_row);
      }
      bool operator==(const iterator& _rhs) const
      {
        return m_row == _rhs.m_row and m_table_ptr == _rhs.m_table_ptr;
      }
      bool operator!=(const iterator& _rhs) const
      {
        return m_row != _rhs.m_row or m_table_ptr != _rhs.m_table_ptr;
      }
      iterator operator+(int _inc) const
      {
        return iterator{m_row + _inc, m_table_ptr};
      }
      iterator operator-(int _inc) const
      {
        return iterator{m_row - _inc, m_table_ptr};
      }
      std::ptrdiff_t operator-(const iterator& _rhs) const
      {
        errors::check_condition(m_table_ptr == _rhs.m_table_ptr, "Iterator arithmetic on iterator for different tables is not allowed.");
        return m_row - std::ptrdiff_t(_rhs.m_row);
      }
      bool operator<(const iterator& _rhs) const
      {
        errors::check_condition(m_table_ptr == _rhs.m_table_ptr, "Iterator arithmetic on iterator for different tables is not allowed.");
        return m_row < _rhs.m_row;
      }
    private:
      std::size_t m_row;
      table* m_table_ptr;
    };
  public:
    table();
    ~table();
  public:
    iterator begin()
    {
      return iterator{0, this};
    }
    iterator end()
    {
      return iterator{m_rows, this};
    }
  public:
    /**
     * Remove the @p _offset first rows of the table.
     */
    void offset(std::size_t _offset);
    /**
     * Resize to maximum @p _limit rows.
     */
    void limit(std::size_t _limit);
    /**
     * Sort the table according to @p _rows. Using quick sort algorithm.
     */
    void sort(const std::vector<std::size_t>& _rows);
    template<typename _CompareDifferent_, typename _CompareOrder_>
    void sort(const std::vector<std::size_t>& _rows, _CompareDifferent_ _cdiff, _CompareOrder_ _corder);
    /**
     * Sort in descending order the table according to @p _rows. Using quick sort algorithm.
     */
    void reverse_sort(const std::vector<std::size_t>& _rows);
    /**
     * Add a single column. Adding column to a table with data is a costly operation.
     */
    void add_column(const std::string& _column, const _T_& _val = _T_());
    /**
     * Add a set of columns. Adding column to a table with data is a costly operation.
     */
    void add_columns(const std::vector<std::string>& _columns, const _T_& _val = _T_());
    /**
     * Add a row to the table.
     */
    void add_row(const std::vector<_T_>& _row);
    /**
     * Set the table to equals the content of the vector.
     * If the size of \ref _table is not a multiple of columns, an exception is thrown.
     */
    void set_rows(const std::vector<_T_>& _table);
    bool has_column(const std::string& _column) const;
    std::size_t get_column_index(const std::string& _column) const;
    /**
     * @return the name of the columns
     */
    std::vector<std::string> get_columns_names() const;
    std::size_t get_columns_count() const;
    std::size_t get_rows_count() const;
    /**
     * @return the row at index \p _j
     */
    const_row_view get_row(std::size_t _j) const;
    /**
     * @return the row at index \p _j
     */
    row_view get_row(std::size_t _j);
    /**
     * @return the value at column \p _i and row \p _j
     */
    const _T_& get_value(std::size_t _i, std::size_t _j) const;
    /**
     * @return the value at column \p _column and row \p _j
     */
    const _T_& get_value(const std::string& _column, std::size_t _j) const;
    bool operator==(const table& _rhs)
    {
      return m_columns == _rhs.m_columns and m_rows == _rhs.m_rows and m_labels == _rhs.m_labels and m_values == _rhs.m_values;
    }
  private:
    template<typename _Compare_>
    void quick_sort(_Compare_ _comp, std::size_t _low, std::size_t _high);
    template<typename _Compare_>
    std::size_t quick_sort_partition(_Compare_ _comp, std::size_t _low, std::size_t _high);
  private:
    void add_columns(std::size_t _count, const _T_& _val);
    std::size_t m_columns = 0, m_rows = 0;
    std::unordered_map<std::string, int> m_labels;
    std::vector<_T_> m_values;
  };

  template<typename _T_>
  inline table<_T_>::table() {}
  template<typename _T_>
  inline table<_T_>::~table() {}

  template<typename _T_>
  inline void table<_T_>::offset(std::size_t _offset)
  {
    if(_offset >= m_rows)
    {
      m_values.clear();
      m_rows = 0;
    } else {
      m_rows -= _offset;
      m_values.erase(m_values.begin(), m_values.begin() + _offset * m_columns);
    }
  }
  template<typename _T_>
  inline void table<_T_>::limit(std::size_t _limit)
  {
    m_rows = std::min(m_rows, _limit);
    m_values.resize(m_columns * m_rows);
  }
  template<typename _T_>
  template<typename _Compare_>
  void table<_T_>::quick_sort(_Compare_ _comp, std::size_t _low, std::size_t _high)
  {
    if(_low < get_rows_count() and _high < get_rows_count() and _low < _high)
    {
      std::size_t p = quick_sort_partition(_comp, _low, _high); 
      quick_sort(_comp, _low, p); // Note: the pivot is now included
      quick_sort(_comp, p + 1, _high); 
    }
  }
  template<typename _T_>
  template<typename _Compare_>
  std::size_t table<_T_>::quick_sort_partition(_Compare_ _comp, std::size_t _low, std::size_t _high)
  {
    // Pivot value
    std::vector<_T_> pivot_v = get_row((_high - _low)/2 + _low); // The value in the middle of the array
    row_view pivot(pivot_v.begin(), pivot_v.end());
    // Left index
    std::size_t i = _low - 1; 

    // Right index
    std::size_t j = _high + 1;

    while(true)
    {
      // Move the left index to the right at least once and while the element at
      // the left index is less than the pivot
      do { ++i; } while(_comp(get_row(i), pivot));
      
      // Move the right index to the left at least once and while the element at
      // the right index is greater than the pivot
      do { --j; } while(_comp(pivot, get_row(j)));


      // If the indices crossed, return
      if(i >= j) { return j; }
      // Swap the elements at the left and right indices
      swap_span_content(get_row(i), get_row(j));
    }
  }
  template<typename _T_>
  template<typename _CompareDifferent_, typename _CompareOrder_>
  void table<_T_>::sort(const std::vector<std::size_t>& _rows, _CompareDifferent_ _cdiff, _CompareOrder_ _corder)
  {
    // cannot use std::sort, as it requires iterator to support movable, which table can't since they return a span
    quick_sort([_rows, _cdiff, _corder](const const_row_view& it1, const const_row_view& it2)
    {
      for(std::size_t r : _rows)
      {
        errors::check_condition(r < it1.size(), "Invalid row index {}", r);
        const _T_& v1 = it1[r];
        const _T_& v2 = it2[r];
        if(_cdiff(v1, v2))
        {
          return _corder(v1, v2);
        }
      }
      return false;
    }, 0, get_rows_count() - 1);
  }
  template<typename _T_>
  inline void table<_T_>::sort(const std::vector<std::size_t>& _rows)
  {
    // cannot use std::sort, as it requires iterator to support movable, which table can't since they return a span
    return sort(_rows, std::not_equal_to(), std::less());
  }
  template<typename _T_>
  inline void table<_T_>::reverse_sort(const std::vector<std::size_t>& _rows)
  {
    // cannot use std::sort, as it requires iterator to support movable, which table can't since they return a span
    return sort(_rows, std::not_equal_to(), std::greater());
  }
  template<typename _T_>
  inline void table<_T_>::add_column(const std::string& _column, const _T_& _val)
  {
    add_columns(1, _val);
    m_labels[_column] = m_labels.size();
  }
  template<typename _T_>
  inline void table<_T_>::add_columns(const std::vector<std::string>& _columns, const _T_& _val)
  {
    add_columns(_columns.size(), _val);
    for(const std::string& s : _columns)
    {
      m_labels[s] = m_labels.size();
    }
  }
  template<typename _T_>
  inline void table<_T_>::add_row(const std::vector<_T_>& _row)
  {
    errors::check_condition(_row.size() == get_columns_count(), "Invalid row size, got {}, expected {}", _row.size(), get_columns_count());
    m_values.insert(m_values.end(), _row.begin(), _row.end());
    ++m_rows;
  }
  template<typename _T_>
  inline void table<_T_>::set_rows(const std::vector<_T_>& _table)
  {
    errors::check_condition(_table.size() % get_columns_count() == 0, "Invalid table size {}, should be a multiple of columns {}.", _table.size(), get_columns_count());
    m_values = _table;
    m_rows = _table.size() / m_columns;
  }
  template<typename _T_>
  bool table<_T_>::has_column(const std::string& _column) const
  {
    return m_labels.find(_column) != m_labels.end();
  }
  template<typename _T_>
  std::size_t table<_T_>::get_column_index(const std::string& _column) const
  {
    auto it = m_labels.find(_column);
    if(it != m_labels.end()) return it->second;
    throw exception("Unknown column {}");
  }
  template<typename _T_>
  inline std::vector<std::string> table<_T_>::get_columns_names() const
  {
    std::vector<std::string> col;
    col.resize(m_columns);
    for(auto const& [k, v] : m_labels)
    {
      col[v] = k;
    }
    return col;
  }
  template<typename _T_>
  inline std::size_t table<_T_>::get_columns_count() const
  {
    return m_columns;
  }
  template<typename _T_>
  inline std::size_t table<_T_>::get_rows_count() const
  {
    return m_rows;
  }
  template<typename _T_>
  inline table<_T_>::const_row_view table<_T_>::get_row(std::size_t _j) const
  {
    errors::check_condition(_j < m_rows, "Invalid index {}, size is {}", _j, m_rows);
    return const_row_view(m_values.begin() + _j * m_columns, m_values.begin() + (_j+1) * m_columns);
  }
  template<typename _T_>
  inline table<_T_>::row_view table<_T_>::get_row(std::size_t _j)
  {
    errors::check_condition(_j < m_rows, "Invalid index {}, size is {}", _j, m_rows);
    return row_view(m_values.begin() + _j * m_columns, m_values.begin() + (_j+1) * m_columns);
  }
  template<typename _T_>
  inline const _T_& table<_T_>::get_value(std::size_t _i, std::size_t _j) const
  {
    return m_values[_i + _j * m_columns];
  }
  template<typename _T_>
  inline const _T_& table<_T_>::get_value(const std::string& _column, std::size_t _j) const
  {
    auto it = m_labels.find(_column);
    if(it == m_labels.end()) throw gqlite::exception("Unknown column {}", _column);
    return get_value(it->second, _j);
  }
  template<typename _T_>
  inline void table<_T_>::add_columns(std::size_t _count, const _T_& _val)
  {
    std::size_t new_columns = m_columns + _count;
    if(m_rows > 0)
    {
      std::vector<_T_> new_values;
      new_values.reserve(new_columns * m_rows);
      auto new_value_bi = std::back_inserter(new_values);
      for(std::size_t j = 0; j < m_rows; ++j)
      {
        std::copy(m_values.begin() + m_columns * j, m_values.begin()  + m_columns * (j+1), new_value_bi);
        std::fill_n(new_value_bi, _count, _val);
      }
      m_values = new_values;
    }
    m_columns = new_columns;
  }
}
