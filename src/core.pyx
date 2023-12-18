#cython: language_level=3
from libcpp.vector cimport vector
from libcpp.string cimport string

cdef extern from "phonemizer.cc":
  cdef cppclass Phonemizer:
    Phonemizer(string&) except +
    vector[string] phoneticize(string&)
    
cdef class Model:
  cdef Phonemizer* phonemizer

  def __cinit__(self, str modelfile):
    self.phonemizer = new Phonemizer(modelfile.encode("utf-8"))
  
  def phoneticize(self, str value):
    cdef string c_value = value.encode("utf-8")
    return [c.decode("utf-8") for c in self.phonemizer.phoneticize(c_value)]

  def __dealloc__(self):
    del self.phonemizer